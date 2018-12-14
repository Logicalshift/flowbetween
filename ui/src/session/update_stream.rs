use super::core::*;
use super::state::*;
use super::update::*;
use super::canvas_stream::*;
use super::viewmodel_stream::*;
use super::super::diff::*;
use super::super::control::*;
use super::super::controller::*;

use desync::*;
use binding::*;
use futures::*;
use futures::task::Task;

use std::mem;
use std::sync::*;

///
/// Core data for an update stream
/// 
struct UpdateStreamCore {
    /// The state of the UI last time an update was generated for the update stream
    state: UiSessionState,

    /// The ID of the last update that was generated
    last_update_id: u64,

    /// Task that's waiting for a pending update
    waiting: Option<Task>
}

///
/// Stream that can be used to retrieve the most recent set of UI updates from
/// the core. It's possible to retrieve empty updates in the event the core processed
/// events that produced no changes (ie, sending an event to the sink will cause this
/// stream to eventually return at least one update set)
/// 
/// Every update stream begins with an update that sets the initial state of the
/// UI.
/// 
pub struct UiUpdateStream {
    /// The session core
    session_core: Arc<Desync<UiSessionCore>>,

    /// The stream core
    stream_core: Arc<Desync<UpdateStreamCore>>,

    /// The UI tree for the core controller
    _ui_tree: BindRef<Control>,

    /// The state of the UI at the last update
    last_ui: Option<Control>,

    /// The updates for the UI tree
    ui_updates: FollowStream<Control, BindRef<Control>>,

    /// The viewmodel updates
    viewmodel_updates: ViewModelUpdateStream,

    // The canvas updates
    canvas_updates: CanvasUpdateStream,

    /// Pending updates from the UI (these have priority as we want the UI updates to happen first, but we poll them last)
    pending_ui: Arc<Mutex<Option<Vec<UiUpdate>>>>,

    /// Pending updates from the canvases and the viewmodels
    pending: Arc<Mutex<Option<Vec<UiUpdate>>>>,
}

impl UiUpdateStream {
    ///
    /// Creates a new UI update stream
    /// 
    pub fn new(controller: Arc<dyn Controller>, core: Arc<Desync<UiSessionCore>>) -> UiUpdateStream {
        // Create the values that will go into the core
        let session_core        = core;
        let stream_core         = Arc::new(Desync::new(UpdateStreamCore::new()));
        let pending             = Arc::new(Mutex::new(None));
        let pending_ui          = Arc::new(Mutex::new(Some(vec![UiUpdate::Start])));

        // Stream from the ui
        let ui_tree             = assemble_ui(Arc::clone(&controller));
        let ui_updates          = follow(ui_tree.clone());

        // Stream from the viewmodel
        let viewmodel_updates   = ViewModelUpdateStream::new(Arc::clone(&controller));

        // Stream from the canvases
        let canvas_updates      = CanvasUpdateStream::new(Arc::clone(&controller));
        
        // Generate the stream
        let new_stream = UiUpdateStream {
            session_core:       session_core,
            stream_core:        stream_core,
            _ui_tree:           ui_tree,
            ui_updates:         ui_updates,
            last_ui:            None,
            viewmodel_updates:  viewmodel_updates,
            canvas_updates:     canvas_updates,
            pending_ui:         pending_ui,
            pending:            pending
        };

        new_stream
    }

    ///
    /// Creates the initial set of pending events (initial UI refresh and viewmodel refresh)
    /// 
    fn generate_initial_event(&mut self) {
        let session_core    = Arc::clone(&self.session_core);
        let stream_core     = Arc::clone(&self.stream_core);
        let pending         = Arc::clone(&self.pending);

        session_core.desync(move |session_core| {
            let update_id  = session_core.last_update_id();
            let ui_binding = session_core.ui_tree();

            stream_core.desync(move |stream_core| {
                // Get the initial UI tree
                let ui_tree = ui_binding.get();

                // We generate an update that sends the entire UI and viewmodel state to the target
                let initial_ui          = stream_core.state.update_ui(&ui_tree);

                // Turn into a set of updates
                // These updates include the start event
                let mut updates = vec![UiUpdate::Start];
                if let Some(initial_ui) = initial_ui { updates.push(initial_ui); }

                // This is the initial pending update
                let mut pending = pending.lock().unwrap();

                *pending = Some(updates);

                // Set the update ID where this was triggered
                stream_core.last_update_id = update_id;

                // Poke anything that's waiting for an update
                let mut waiting = None;
                mem::swap(&mut waiting, &mut stream_core.waiting);
                waiting.map(|waiting| waiting.notify());
            });
        })
    }

    ///
    /// Pulls any UI events into the pending stream
    ///
    fn pull_ui_events(&mut self) {
        // Pending UI updates
        let mut ui_updates = vec![];

        // Poll for as many updates as there are
        while let Ok(Async::Ready(Some(new_ui))) = self.ui_updates.poll() {
            if let Some(last_ui) = self.last_ui.take() {
                // Find the differences in the UI
                let differences = diff_tree(&last_ui, &new_ui);

                if differences.len() != 0 {
                    // Found some differences: change into a series of UiDiffs
                    let diffs = differences.into_iter()
                        .map(|diff| UiDiff {
                            address:    diff.address().clone(),
                            new_ui:     diff.replacement().clone()
                        })
                        .collect::<Vec<_>>();
                    
                    ui_updates.extend(diffs);
                }

                // The new UI is now the last UI
                self.last_ui = Some(new_ui);
            } else {
                // Create a diff from the entire UI
                ui_updates.push(UiDiff {
                    address:    vec![],
                    new_ui:     new_ui.clone()
                });

                // This is now the last UI
                self.last_ui = Some(new_ui);
            }
        }

        if ui_updates.len() > 0 {
            self.pending_ui.lock().unwrap()
                .get_or_insert_with(|| vec![])
                .push(UiUpdate::UpdateUi(ui_updates))
        }
    }

    ///
    /// Pulls any viewmodel events into the pending stream
    ///
    fn pull_viewmodel_events(&mut self) {
        // Pending viewmodel updates
        let mut viewmodel_updates = vec![];

        // For as long as the viewmodel stream has updates, add them to the viewmodel update list
        while let Ok(Async::Ready(Some(update))) = self.viewmodel_updates.poll() {
            viewmodel_updates.push(update);
        }

        // Add a viewmodel update to the pending list if there were any
        if viewmodel_updates.len() > 0 {
            self.pending.lock().unwrap()
                .get_or_insert_with(|| vec![])
                .push(UiUpdate::UpdateViewModel(viewmodel_updates));
        }
    }

    ///
    /// Pulls any viewmodel events into the pending stream
    ///
    fn pull_canvas_events(&mut self) {
        // Pending canvas updates
        let mut canvas_updates = vec![];

        // For as long as the canvas stream has updates, add them to the viewmodel update list
        while let Ok(Async::Ready(Some(update))) = self.canvas_updates.poll() {
            canvas_updates.push(update);
        }

        // Add a canvas update to the pending list if there were any
        if canvas_updates.len() > 0 {
            self.pending.lock().unwrap()
                .get_or_insert_with(|| vec![])
                .push(UiUpdate::UpdateCanvas(canvas_updates));
        }
    }
}

impl UpdateStreamCore {
    ///
    /// Creates a new update stream core
    /// 
    pub fn new() -> UpdateStreamCore {
        UpdateStreamCore {
            state:          UiSessionState::new(),
            last_update_id: 0,
            waiting:        None
        }
    }

    ///
    /// Updates against a pending update
    /// 
    pub fn finish_update(&mut self, ui_binding: &BindRef<Control>, update_id: u64, pending: Arc<Mutex<Option<Vec<UiUpdate>>>>) {
        if update_id == self.last_update_id {
            // Already dispatched this update
            return;
        }

        let mut pending = pending.lock().unwrap();
        if pending.is_some() {
            // Different update is already pending
            return;
        }

        if let Some(ref waiting) = self.waiting {
            // Something is waiting for an update, so we're going to generate it now
            let update  = self.state.get_updates(ui_binding);
            *pending    = Some(update);

            // Poke whatever's waiting to let it know that its update has arrived
            self.last_update_id = update_id;
            waiting.notify();
        }
    }
}

impl Stream for UiUpdateStream {
    type Item   = Vec<UiUpdate>;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<Vec<UiUpdate>>, Self::Error> {
        // Pull any pending events into the pending list
        self.pull_canvas_events();
        self.pull_viewmodel_events();

        // UI events are polled last but we return them first (this way the viewmodel and canvas updates apply to the current UI)
        self.pull_ui_events();

        // Try to read the pending update, if there is one
        let mut pending_ui          = self.pending_ui.lock().unwrap();
        let mut pending             = self.pending.lock().unwrap();
        let mut pending_result_ui   = None;
        let mut pending_result      = None;

        mem::swap(&mut pending_result_ui, &mut *pending_ui);
        mem::swap(&mut pending_result, &mut *pending);

        // The UI updates are always performed before the canvas and viewmodel updates
        if let Some(mut pending_result_ui) = pending_result_ui {
            pending_result_ui.extend(pending_result.take().unwrap_or(vec![]));
            pending_result = Some(pending_result_ui);
        }
        
        // Result is OK if we found a pending update
        if let Some(pending) = pending_result {
            // There is a pending update
            Ok(Async::Ready(Some(pending)))
        } else {
            // Not ready yet
            Ok(Async::NotReady)
        }
    }
}

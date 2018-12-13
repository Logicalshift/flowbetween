use super::core::*;
use super::state::*;
use super::update::*;
use super::viewmodel_stream::*;
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

    /// The viewmodel updates
    viewmodel_updates: ViewModelUpdateStream,

    /// Update that was generated for the last poll and is ready to go
    pending: Arc<Mutex<Option<Vec<UiUpdate>>>>,
}

impl UiUpdateStream {
    ///
    /// Creates a new UI update stream
    /// 
    pub fn new(controller: Arc<dyn Controller>, core: Arc<Desync<UiSessionCore>>) -> UiUpdateStream {
        // Create the values that will go into the core
        let session_core    = core;
        let stream_core     = Arc::new(Desync::new(UpdateStreamCore::new()));
        let pending         = Arc::new(Mutex::new(None));

        // Set up the core to receive updates
        Self::initialise_core(Arc::clone(&session_core), Arc::clone(&stream_core));

        // Stream from the viewmodel
        let viewmodel_updates = ViewModelUpdateStream::new(Arc::clone(&controller));
        
        // Generate the stream
        let mut new_stream = UiUpdateStream {
            session_core:       session_core,
            stream_core:        stream_core,
            viewmodel_updates:  viewmodel_updates,
            pending:            pending
        };

        // Send the setup event to it
        new_stream.generate_initial_event();

        new_stream
    }

    ///
    /// Sets up the stream core with its initial state
    /// 
    fn initialise_core(session_core: Arc<Desync<UiSessionCore>>, stream_core: Arc<Desync<UpdateStreamCore>>) {
        session_core.desync(move |session_core| {
            // Need the UI binding from the core
            let ui_binding = session_core.ui_tree();

            // Set up the core with its initial state
            stream_core.desync(move |stream_core| {
                stream_core.setup_state(&ui_binding);
            })
        })
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
    /// Sets up the state object to track updates
    /// 
    pub fn setup_state(&mut self, ui_binding: &BindRef<Control>) {
        self.state.watch_canvases(ui_binding);
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
        // Pull any pending viewmodel events into the pending list
        self.pull_viewmodel_events();

        // Try to read the pending update, if there is one
        let mut pending         = self.pending.lock().unwrap();
        let mut pending_result  = None;

        mem::swap(&mut pending_result, &mut *pending);
        
        // Result is OK if we found a pending update
        if let Some(pending) = pending_result {
            // There is a pending update
            Ok(Async::Ready(Some(pending)))
        } else {
            // No update available yet. We need to register with the core to trigger one
            let task                = task::current();
            let pending             = Arc::clone(&self.pending);
            let session_core        = Arc::clone(&self.session_core);
            let stream_core         = Arc::clone(&self.stream_core);
            let update_pending      = Arc::clone(&self.pending);
            let update_stream_core  = Arc::clone(&self.stream_core);

            session_core.desync(move |session_core| {
                stream_core.sync(move |stream_core| {
                    let mut pending = pending.lock().unwrap();

                    if pending.is_some() {
                        // If there's now a pending update, then signal the task to return via the stream
                        task.notify();
                    } 

                    if session_core.last_update_id() != stream_core.last_update_id {
                        // If the core has a newer update than we do then start generating a new pending update
                        pending.get_or_insert_with(|| vec![])
                            .extend(stream_core.state.get_updates(&session_core.ui_tree()));

                        stream_core.last_update_id = session_core.last_update_id();
                        task.notify();
                    } else {
                        // Otherwise, ask the core to notify us when an update is available
                        stream_core.waiting = Some(task);
                        let ui_binding      = session_core.ui_tree();

                        session_core.on_next_update(move |session_core| {
                            let this_update_id = session_core.last_update_id();

                            update_stream_core.desync(move |stream_core| stream_core.finish_update(&ui_binding, this_update_id, update_pending));
                        });
                    }
                });
            });

            // Not ready yet
            Ok(Async::NotReady)
        }
    }
}

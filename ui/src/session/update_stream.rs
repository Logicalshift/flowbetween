use super::update::*;
use super::canvas_stream::*;
use super::viewmodel_stream::*;
use super::super::diff::*;
use super::super::control::*;
use super::super::controller::*;

use flo_binding::*;
use flo_stream::*;

use futures::*;
use futures::task::{Poll, Context};

use std::mem;
use std::pin::*;
use std::sync::*;

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
    /// The UI tree for the core controller
    _ui_tree: BindRef<Control>,

    /// The state of the UI at the last update
    last_ui: Option<Control>,

    /// Stream of 'update suspend' events
    update_suspend: Subscriber<bool>,

    /// Stream of ticks
    tick: Subscriber<()>,

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

    /// Number of times the update stream has been suspended
    is_suspended: bool
}

impl UiUpdateStream {
    ///
    /// Creates a new UI update stream
    ///
    pub fn new(controller: Arc<dyn Controller>, tick: Subscriber<()>, update_suspend: Subscriber<bool>) -> UiUpdateStream {
        // Create the values that will go into the core
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
            _ui_tree:           ui_tree,
            ui_updates:         ui_updates,
            last_ui:            None,
            update_suspend:     update_suspend,
            viewmodel_updates:  viewmodel_updates,
            canvas_updates:     canvas_updates,
            pending_ui:         pending_ui,
            pending:            pending,
            tick:               tick,
            is_suspended:       false
        };

        new_stream
    }

    ///
    /// Pulls any UI events into the pending stream
    ///
    fn pull_ui_events(&mut self, context: &mut Context) {
        // Pending UI updates
        let mut ui_updates = vec![];

        // Poll for as many updates as there are
        while let Poll::Ready(Some(new_ui)) = self.ui_updates.poll_next_unpin(context) {
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
    fn pull_viewmodel_events(&mut self, context: &mut Context) {
        // Pending viewmodel updates
        let mut viewmodel_updates = vec![];

        // For as long as the viewmodel stream has updates, add them to the viewmodel update list
        while let Poll::Ready(Some(update)) = self.viewmodel_updates.poll_next_unpin(context) {
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
    fn pull_canvas_events(&mut self, context: &mut Context) {
        // Pending canvas updates
        let mut canvas_updates = vec![];

        // For as long as the canvas stream has updates, add them to the viewmodel update list
        while let Poll::Ready(Some(update)) = self.canvas_updates.poll_next_unpin(context) {
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

impl Stream for UiUpdateStream {
    type Item   = Result<Vec<UiUpdate>, ()>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        let self_ref = self.get_mut();

        // Check for suspensions (poll until the update queue goes to pending as we want to return 'pending')
        while let Poll::Ready(Some(is_suspended)) = self_ref.update_suspend.poll_next_unpin(context) {
            self_ref.is_suspended = is_suspended;
        }

        if self_ref.is_suspended {
            // Stay 'not ready' for as long as we're suspended for
            Poll::Pending
        } else {
            // Pull any pending events into the pending list
            self_ref.pull_canvas_events(context);
            self_ref.pull_viewmodel_events(context);

            // UI events are polled last but we return them first (this way the viewmodel and canvas updates apply to the current UI)
            self_ref.pull_ui_events(context);

            // Try to read the pending update, if there is one
            let mut pending_ui          = self_ref.pending_ui.lock().unwrap();
            let mut pending             = self_ref.pending.lock().unwrap();
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
                Poll::Ready(Some(Ok(pending)))
            } else if let Poll::Ready(Some(())) = self_ref.tick.poll_next_unpin(context) {
                // There is a pending tick
                Poll::Ready(Some(Ok(vec![])))
            } else {
                // Not ready yet
                Poll::Pending
            }
        }
    }
}

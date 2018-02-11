use super::update::*;
use super::canvas_state::*;
use super::super::diff::*;
use super::super::control::*;
use super::super::controller::*;
use super::super::diff_viewmodel::*;

use binding::*;

use std::mem;
use std::sync::*;

///
/// Represents the most recent state of a UI session
/// 
pub struct UiSessionState {
    /// The control state at the last update
    ui: Option<Control>,

    /// Creates watchers for the viewmodel
    view_model_diff: Option<DiffViewModel>,

    /// The view model watcher
    view_model_watcher: Option<WatchViewModel>,

    /// The canvas state object
    canvas_state: Option<CanvasState>
}

impl UiSessionState {
    ///
    /// Creates a new UI session state (which initially stores no state)
    /// 
    pub fn new() -> UiSessionState {
        UiSessionState {
            ui:                 None,
            view_model_diff:    None,
            view_model_watcher: None,
            canvas_state:       None
        }
    }

    ///
    /// Updates the UI for this element, returning the corresponding update event
    /// 
    pub fn update_ui(&mut self, new_ui: &Control) -> Option<UiUpdate> {
        let update = if let Some(ref old_ui) = self.ui {
            // Find the differences for the UI
            let differences = diff_tree(old_ui, new_ui);

            if differences.len() == 0 {
                // UI has not changed
                None
            } else {
                // Found some differences: change into a series of UiDiffs
                let diffs = differences.into_iter()
                    .map(|diff| UiDiff {
                        address:    diff.address().clone(),
                        new_ui:     diff.replacement().clone()
                    })
                    .collect();
                
                Some(UiUpdate::UpdateUi(diffs))
            }
        } else {
            // Generate an update with the entire UI
            Some(UiUpdate::UpdateUi(vec![UiDiff {
                address:    vec![],
                new_ui:     new_ui.clone()
            }]))
        };

        // Update the UI
        if update.is_some() {
            self.ui = Some(new_ui.clone());
        }

        // The update is the result
        update
    }

    ///
    /// Starts watching the viewmodel for a controller
    /// 
    pub fn watch_viewmodel(&mut self, controller: Arc<Controller>) {
        let new_diff    = DiffViewModel::new(controller);
        let watcher     = new_diff.watch();

        self.view_model_diff    = Some(new_diff);
        self.view_model_watcher = Some(watcher);
    }

    ///
    /// Retrieves the viewmodel update event, if there is one
    /// 
    pub fn get_viewmodel_update(&mut self) -> Option<UiUpdate> {
        // Pull the watcher out of this object
        let mut watcher = None;
        mem::swap(&mut watcher, &mut self.view_model_watcher);

        if let Some((diff, watcher)) = self.view_model_diff.as_ref().and_then(|diff| watcher.map(move |watch| (diff, watch))) {
            // We're watching a viewmodel; rotate the watch to build the diff
            let (differences, new_watcher) = diff.rotate_watch(watcher);

            // This becomes our new watcher
            self.view_model_watcher = Some(new_watcher);

            // No event if there are no differences, otherwise a viewmodel change update
            if differences.len() == 0 {
                None
            } else {
                Some(UiUpdate::UpdateViewModel(differences))
            }
        } else {
            None
        }
    }

    ///
    /// Watches for updates to canvases in the specified UI
    /// 
    pub fn watch_canvases(&mut self, ui_binding: &BindRef<Control>) {
        self.canvas_state = Some(CanvasState::new(ui_binding));
    }

    ///
    /// Retrieves the updates to any canvases attached to this object
    /// 
    pub fn get_canvas_update(&mut self) -> Option<UiUpdate> {
        if let Some(ref canvas_state) = self.canvas_state {
            // Fetch the updates from the canvas state
            let updates = canvas_state.latest_updates();

            if updates.len() == 0 {
                // No updates
                None
            } else {
                // Got some updates
                Some(UiUpdate::UpdateCanvas(updates))
            }
        } else {
            // Not watching any canvases
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initial_ui_diff_contains_everything() {
        let mut state = UiSessionState::new();

        assert!(state.update_ui(&Control::empty()) == Some(UiUpdate::UpdateUi(vec![UiDiff { address: vec![], new_ui: Control::empty() }])));
    }

    #[test]
    fn update_ui_with_no_difference_returns_no_updates() {
        let mut state = UiSessionState::new();

        state.update_ui(&Control::empty());
        assert!(state.update_ui(&Control::empty()) == None);
    }

    #[test]
    fn changing_ui_generates_differences() {
        let mut state = UiSessionState::new();

        state.update_ui(&Control::empty()
            .with(vec![
                Control::label().with("Test1")
            ]));
        
        assert!(state.update_ui(&Control::empty()
            .with(vec![
                Control::label().with("Test2")
            ])) == Some(UiUpdate::UpdateUi(vec![
                UiDiff {
                    address:    vec![0],
                    new_ui:     Control::label().with("Test2")
                }
            ])));
    }
}
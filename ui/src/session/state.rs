use super::update::*;
use super::canvas_state::*;
use super::super::diff::*;
use super::super::control::*;
use super::super::controller::*;
use super::super::diff_viewmodel::*;

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
    pub fn update_ui(&mut self, new_ui: Control) -> Option<UiUpdate> {
        let update = if let Some(ref old_ui) = self.ui {
            // Find the differences for the UI
            let differences = diff_tree(old_ui, &new_ui);

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
        self.ui = Some(new_ui);

        // The update is the result
        update
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initial_ui_diff_contains_everything() {
        let mut state = UiSessionState::new();

        assert!(state.update_ui(Control::empty()) == Some(UiUpdate::UpdateUi(vec![UiDiff { address: vec![], new_ui: Control::empty() }])));
    }

    #[test]
    fn update_ui_with_no_difference_returns_no_updates() {
        let mut state = UiSessionState::new();

        state.update_ui(Control::empty());
        assert!(state.update_ui(Control::empty()) == None);
    }

    #[test]
    fn changing_ui_generates_differences() {
        let mut state = UiSessionState::new();

        state.update_ui(Control::empty()
            .with(vec![
                Control::label().with("Test1")
            ]));
        
        assert!(state.update_ui(Control::empty()
            .with(vec![
                Control::label().with("Test2")
            ])) == Some(UiUpdate::UpdateUi(vec![
                UiDiff {
                    address: vec![0],
                    new_ui: Control::label().with("Test2")
                }
            ])));
    }
}
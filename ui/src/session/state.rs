use super::update::*;
use super::super::diff::*;
use super::super::control::*;

use flo_binding::*;

///
/// Represents the most recent state of a UI session
///
pub struct UiSessionState {
    /// The control state at the last update
    ui: Option<Control>,
}

impl UiSessionState {
    ///
    /// Creates a new UI session state (which initially stores no state)
    ///
    pub fn new() -> UiSessionState {
        UiSessionState {
            ui:                 None
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
    /// Retrieves the updates that are pending in this object
    ///
    pub fn get_updates(&mut self, ui_binding: &BindRef<Control>) -> Vec<UiUpdate> {
        // Fetch the updates for the various categories
        let ui_updates          = self.update_ui(&ui_binding.get());

        // Combine into a vector
        let mut combined_updates = vec![];
        if let Some(ui_updates) = ui_updates                { combined_updates.push(ui_updates); }

        combined_updates
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

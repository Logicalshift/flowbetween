use crate::scenery::ui::control::*;
use crate::scenery::ui::control_id::*;
use crate::scenery::ui::dialog::*;
use crate::scenery::ui::ui_path::*;

use std::collections::{HashMap};

struct ControlState {
    location:   (UiPoint, UiPoint),
    value:      ControlValue,
    visible:    bool,
}

///
/// The state of an egui dialog
///
pub struct EguiDialogState {
    /// The controls in order
    controls: Vec<(ControlId, ControlType)>,

    /// The state of the controls in this dialog
    states: HashMap<ControlId, ControlState>,
}

impl EguiDialogState {
    ///
    /// Creates a new dialog state
    ///
    pub fn new() -> Self {
        EguiDialogState {
            controls:   vec![],
            states:     HashMap::new(),
        }
    }
    
    ///
    /// Updates this state from a dialog request
    ///
    pub fn update_state(&mut self, request: &Dialog) {
        use Dialog::*;

        match request {
            CreateDialog(_, _, _)   |
            RemoveDialog(_)         |
            MoveDialog(_, _)        => { /* We only deal with control requests */ }

            AddControl(_, control_id, bounds, control_type, initial_value) => {
                // Add to the list of controls and create the initial state
                self.controls.push((*control_id, control_type.clone()));
                self.states.insert(*control_id, ControlState {
                    location:   *bounds,
                    value:      initial_value.clone(),
                    visible:    true,
                });
            }

            SetControlValue(_, control_id, new_value) => {
                if let Some(state) = self.states.get_mut(&control_id) {
                    state.value = new_value.clone();
                }
            }

            MoveControl(_, control_id, new_bounds) => {
                if let Some(state) = self.states.get_mut(&control_id) {
                    state.location = *new_bounds;
                }
            }

            SetVisible(_, control_id, is_visible) => {
                if let Some(state) = self.states.get_mut(&control_id) {
                    state.visible = *is_visible;
                }
            }
        }
    }
}

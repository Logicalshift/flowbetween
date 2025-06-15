use crate::scenery::ui::control::*;
use crate::scenery::ui::control_id::*;
use crate::scenery::ui::dialog::*;
use crate::scenery::ui::ui_path::*;

use egui;

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

    ///
    /// Runs this control state, returning the events that should be sent
    ///
    pub fn run(&mut self, context: &egui::Context) -> Vec<ControlEvent> {
        // Events that are generated for this UI
        let mut events = vec![];

        // Borrow the fields
        let controls    = &mut self.controls;
        let states      = &mut self.states;

        // Render each control in order
        egui::CentralPanel::default().show(&context, |ui| {
            // Render all the controls
            for (control_id, control_type) in controls.iter() {
                // Each control must have a state
                if let Some(control_state) = states.get_mut(&control_id) {
                    use ControlType::*;

                    if control_state.visible {
                        // Select the region the control will be in (we don't use egui's own layout)
                        let pos = egui::Rect {
                            min: egui::Pos2 { x: control_state.location.0.0 as _, y: control_state.location.0.1 as _ },
                            max: egui::Pos2 { x: control_state.location.1.0 as _, y: control_state.location.1.1 as _ },
                        };

                        // Render the control
                        match control_type {
                            Label(label)        => { ui.put(pos, egui::Label::new(label)); },
                            Button(label)       => { if ui.button(label).clicked() { events.push(ControlEvent::Pressed(*control_id)); } }
                            Checkbox(label)     => { },
                            RadioButton(label)  => { },
                            ProgressBar         => { },
                            Spinner             => { },
                            Separator           => { },
                            Slider(range)       => { },
                        }
                    }
                }
            }
        });

        events
    }
}

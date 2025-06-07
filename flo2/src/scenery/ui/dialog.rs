//!
//! The Dialog subprogram provides conventional 'dialog' type user interface elements
//!

use super::control_id::*;

use flo_scene::*;

use serde::*;

///
/// Dialog actions that happen within a document window
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Dialog {

}

///
/// Events received from a dialog
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DialogEvent {
    /// Indicates that a control has been activated
    Activate(ControlId),

    /// Indicates that a control's string value has changed
    SetValueString(ControlId, String),

    /// Indicates that a control's numeric value has changed
    SetValueNumber(ControlId, usize),
}

impl SceneMessage for Dialog {

}

impl SceneMessage for DialogEvent {

}

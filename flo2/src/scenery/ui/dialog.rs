//!
//! The Dialog subprogram provides conventional 'dialog' type user interface elements
//!

use super::control::*;
use super::control_id::*;
use super::dialog::*;
use super::egui::*;
use super::focus::*;
use super::subprograms::*;
use super::ui_path::*;

use flo_scene::*;
use flo_scene::programs::*;

use futures::prelude::*;
use serde::*;

///
/// Low-level actions related to creating dialog boxes
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Dialog {
    /// Event indicating that the scene is idle
    Idle,

    /// Event from the focus subprogram (used to direct events to the dialog program)
    FocusEvent(FocusEvent),

    /// Creates a dialog region in the canvas. Events for the dialog are sent to the supplied subprogram ID.
    CreateDialog(DialogId, SubProgramId, Bounds<UiPoint>),

    /// Removes a dialog from the canvas (dialogs are also removed if the subprogram stops)
    RemoveDialog(DialogId, Bounds<UiPoint>),

    /// Changes the position of a dialog
    MoveDialog(DialogId, Bounds<UiPoint>),

    /// Adds a control to a dialog. Coordinates are relative to the top-left corner of the dialog
    AddControl(DialogId, ControlId, Bounds<UiPoint>, ControlType, ControlValue),

    /// Changes the value of a control
    SetControlValue(DialogId, ControlId, ControlValue),

    /// Moves a control to a new position in the dialog
    MoveControl(DialogId, ControlId, Bounds<UiPoint>),

    /// Sets whether or not a control is visible
    SetVisible(DialogId, ControlId, bool),
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
    fn default_target() -> StreamTarget {
        subprogram_dialog().into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        // Set up filters for the focus events/updates
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|focus_events| focus_events.map(|focus| Dialog::FocusEvent(focus)))), (), StreamId::with_message_type::<FocusEvent>()).ok();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|idle_events| idle_events.map(|_idle: IdleNotification| Dialog::Idle))), (), StreamId::with_message_type::<IdleNotification>()).ok();

        // Create the standard focus subprogram when a message is sent for the first tiem
        init_context.add_subprogram(subprogram_dialog(), dialog_egui, 20);
    }
}

impl SceneMessage for DialogEvent {

}

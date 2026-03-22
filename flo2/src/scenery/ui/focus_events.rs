use super::control_id::*;

use flo_draw::*;
use flo_scene::*;

use serde::*;

///
/// Messages that the focus subprogram can send to focused subprograms
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusEvent {
    /// An event has occurred for the specified control
    Event(Option<ControlId>, DrawEvent),

    /// The specified control ID has received keyboard focus
    Focused(ControlId),

    /// The specified control ID has lost keyboard focus (when focus moves, we unfocus first)
    Unfocused(ControlId),
}

///
/// An event relating to the mouse pointer for a focused control
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusPointerEvent {
    /// A pointer device has changed its state
    Pointer(PointerAction, PointerId, PointerState),
}

///
/// An event relating to the keyboard for a focused control
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusKeyboardEvent {
    /// The user has pressed a key (parameters are scancode and the name of the key that was pressed, if known)
    KeyDown(u64, Option<Key>),

    /// The user has released a key (parameters are scancode and the name of the key that was pressed, if known)
    KeyUp(u64, Option<Key>),
}

///
/// An event relating to the window a focused control is in
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusWindowEvent {
    /// The specified control ID has received keyboard focus
    Focused(ControlId),

    /// The specified control ID has lost keyboard focus (when focus moves, we unfocus first)
    Unfocused(ControlId),

    /// The window has a new scale
    Scale(f64),

    /// Window has a new size
    Resize(f64, f64),

    /// Window was closed
    Closed,
}

fn setup_focus_events(init_context: &impl SceneInitialisationContext) {
    // TODO: convert the individual events to the main FocusEvent
}

impl SceneMessage for FocusEvent {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        setup_focus_events(init_context);
    }
}

impl SceneMessage for FocusPointerEvent {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        setup_focus_events(init_context);
    }
}

impl SceneMessage for FocusKeyboardEvent {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        setup_focus_events(init_context);
    }
}

impl SceneMessage for FocusWindowEvent {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        setup_focus_events(init_context);
    }
}

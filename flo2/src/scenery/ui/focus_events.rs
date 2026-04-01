// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::control_id::*;

use flo_draw::*;
use flo_scene::*;

use serde::*;

///
/// Messages that the focus subprogram can send to focused subprograms
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusEvent {
    /// Pointer event
    Pointer(FocusPointerEvent),

    /// Keyboard event
    Keyboard(FocusKeyboardEvent),

    /// Window event
    Window(FocusWindowEvent),
}

///
/// An event relating to the mouse pointer for a focused control
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusPointerEvent {
    /// A pointer device has changed its state
    Pointer(Option<ControlId>, PointerAction, PointerId, PointerState),

    /// The mouse was released over a control in the specified subprogram
    Drop(SubProgramId, Option<ControlId>),

    /// While being held down, the mouse has moved over a control in the specified subprogram
    Hover(SubProgramId, Option<ControlId>),
}

///
/// An event relating to the keyboard for a focused control
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusKeyboardEvent {
    /// The user has pressed a key (parameters are scancode and the name of the key that was pressed, if known)
    KeyDown(Option<ControlId>, u64, Option<Key>),

    /// The user has released a key (parameters are scancode and the name of the key that was pressed, if known)
    KeyUp(Option<ControlId>, u64, Option<Key>),

    /// The specified control ID has received keyboard focus
    Focused(ControlId),

    /// The specified control ID has lost keyboard focus (when focus moves, we unfocus first)
    Unfocused(ControlId),
}

///
/// An event relating to the window a focused control is in
///
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FocusWindowEvent {
    /// The window has a new scale
    Scale(f64),

    /// Window has a new size
    Resize(f64, f64),

    /// Window was closed
    Closed,
}

fn setup_focus_events(init_context: &impl SceneInitialisationContext) {
    // Convert the specific events to the 'general' event type so programs can receive all the events if they need to
    init_context.connect_programs(StreamSource::Filtered(FilterHandle::conversion_filter::<FocusPointerEvent, FocusEvent>()), (), StreamId::with_message_type::<FocusPointerEvent>()).unwrap();
    init_context.connect_programs(StreamSource::Filtered(FilterHandle::conversion_filter::<FocusKeyboardEvent, FocusEvent>()), (), StreamId::with_message_type::<FocusKeyboardEvent>()).unwrap();
    init_context.connect_programs(StreamSource::Filtered(FilterHandle::conversion_filter::<FocusWindowEvent, FocusEvent>()), (), StreamId::with_message_type::<FocusWindowEvent>()).unwrap();
}

impl FocusPointerEvent {
    ///
    /// Returns this event with a new target
    ///
    pub fn with_target(self, new_target: Option<ControlId>) -> Self {
        match self {
            FocusPointerEvent::Pointer(_, pointer_action, pointer_id, pointer_state)    => FocusPointerEvent::Pointer(new_target, pointer_action, pointer_id, pointer_state),
            FocusPointerEvent::Drop(program_id, target_control)                         => FocusPointerEvent::Drop(program_id, target_control),
            FocusPointerEvent::Hover(program_id, target_control)                        => FocusPointerEvent::Hover(program_id, target_control),
        }
    }
}

impl FocusKeyboardEvent {
    ///
    /// Returns this event with a new target
    ///
    pub fn with_target(self, new_target: Option<ControlId>) -> Self {
        match self {
            FocusKeyboardEvent::KeyDown(_, code, key)   => { FocusKeyboardEvent::KeyDown(new_target, code, key) },
            FocusKeyboardEvent::KeyUp(_, code, key)     => { FocusKeyboardEvent::KeyUp(new_target, code, key) },
            FocusKeyboardEvent::Focused(_)              => { FocusKeyboardEvent::Focused(new_target.unwrap()) },
            FocusKeyboardEvent::Unfocused(_)            => { FocusKeyboardEvent::Unfocused(new_target.unwrap()) },
        }
    }
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

impl Into<FocusEvent> for FocusPointerEvent {
    #[inline]
    fn into(self) -> FocusEvent {
        FocusEvent::Pointer(self)
    }
}

impl Into<FocusEvent> for FocusKeyboardEvent {
    #[inline]
    fn into(self) -> FocusEvent {
        FocusEvent::Keyboard(self)
    }
}

impl Into<FocusEvent> for FocusWindowEvent {
    #[inline]
    fn into(self) -> FocusEvent {
        FocusEvent::Window(self)
    }
}

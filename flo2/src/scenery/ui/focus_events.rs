use super::control_id::*;

use flo_draw::*;

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

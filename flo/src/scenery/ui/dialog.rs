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

//!
//! The Dialog subprogram provides conventional 'dialog' type user interface elements
//!

use super::control::*;
use super::dialog_id::*;
use super::egui::*;
use super::subprograms::*;

use flo_scene::*;
use flo_ui::subprograms::*;
use flo_ui::util::*;

use serde::*;

///
/// Low-level actions related to creating dialog boxes
///
#[derive(Serialize, Deserialize, Clone)]
pub enum Dialog {
    /// Creates a dialog region in the canvas. Events for the dialog are sent to the supplied subprogram ID.
    CreateDialog(DialogId, SubProgramId, (UiPoint, UiPoint)),

    /// Removes a dialog from the canvas (dialogs are also removed if the subprogram stops)
    RemoveDialog(DialogId),

    /// Changes the position of a dialog
    MoveDialog(DialogId, (UiPoint, UiPoint)),

    /// Adds a control to a dialog. Coordinates are relative to the top-left corner of the dialog
    AddControl(DialogId, ControlId, (UiPoint, UiPoint), ControlType, ControlValue),

    /// Changes the value of a control
    SetControlValue(DialogId, ControlId, ControlValue),

    /// Moves a control to a new position in the dialog
    MoveControl(DialogId, ControlId, (UiPoint, UiPoint)),

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
    SetValueNumber(ControlId, i64),
}

impl SceneMessage for Dialog {
    fn default_target() -> StreamTarget {
        subprogram_dialog().into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.add_subprogram(subprogram_dialog(), dialog_egui_hub, 20);
    }
}

impl SceneMessage for DialogEvent {

}

use super::control_id::*;
use super::subprograms::*;
use super::ui_path::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::*;
use flo_curves::bezier::path::*;

use futures::prelude::*;
use serde::*;

///
/// Requests to the focus subprogram.
///
/// The focus subprogram deals with mapping mouse clicks on a document window to the subprogram responsible for
/// processing them, as well as routing keyboard events to the control that currently has focus.
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Focus {
    /// An event in need of routing
    Event(DrawEvent),

    /// An update from the scene (used to track when subprograms go away)
    Update(SceneUpdate),

    /// Sets the subprogram that should process keyboard events
    SetKeyboardFocus(SubProgramId, ControlId),

    /// Sets which control should receive keyboard focus after the specified control (within a subprogram, which might have several controls)
    SetFollowingControl(SubProgramId, ControlId, ControlId),

    /// Sets which subprogram should receive keyboard focus after reaching the end of the controls in the specified 
    SetFollowingSubProgram(SubProgramId, SubProgramId),

    /// Move keyboard focus to the next control
    FocusNext,

    /// Move keyboard focus to the preceding control
    FocusPrevious,

    /// Claims a region inside the specified path as belonging to the specified subprogram. The z-index is used to disambiguate requests if more than region matches
    /// Clicks in this region will have 'None' as the control ID
    ClaimRegion { program: SubProgramId, region: Vec<UiPath>, z_index: usize },

    /// Claims a region for a single control within the region for a subprogram. The z-index here is used 
    ClaimControlRegion { program: SubProgramId, region: Vec<UiPath>, control: ControlId, z_index: usize },

    /// Removes a claim added by ClaimRegion
    RemoveClaim(SubProgramId),

    /// Removes a claim added by ClaimControlRegion
    RemoveControlClaim(SubProgramId, ControlId),
}

///
/// Messages that the focus subprogram can send to focused subprograms
///
pub enum FocusEvent {
    /// An event has occurred for the specified control
    Event(Option<ControlId>, DrawEvent),

    /// The specified control ID has received keyboard focus
    Focused(ControlId),
}

impl SceneMessage for Focus {
    fn default_target() -> StreamTarget {
        subprogram_focus().into()
    }
}

///
/// Runs the UI focus subprogram
///
pub async fn focus(input: InputStream<Focus>, context: SceneContext) {
    let mut input = input;

    while let Some(request) = input.next().await {
        // TODO
    }
}

use flo_scene::*;

///
/// ID of the main 'focus' subprogram
///
pub fn subprogram_focus() -> SubProgramId { SubProgramId::called("flowbetween::ui::focus") }

///
/// ID of the main 'focus' subprogram
///
pub fn subprogram_dialog() -> SubProgramId { SubProgramId::called("flowbetween::ui::dialog") }

///
/// ID of the tool state subprogram
///
pub fn subprogram_tool_state() -> SubProgramId { SubProgramId::called("flowbetween::tool_state") }

///
/// ID of the physics layer subprogram
///
pub fn subprogram_physics_layer() -> SubProgramId { SubProgramId::called("flowbetween::ui::physics_layer") }

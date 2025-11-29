use flo_scene::*;

///
/// ID of the main document subprogram
///
pub fn subprogram_flowbetween_document() -> SubProgramId { SubProgramId::called("flowbetween::document::document") }

///
/// ID of the subprogram where DrawingWindowRequests can be sent to
///
pub fn subprogram_window() -> SubProgramId { SubProgramId::called("flowbetween::document::window") }

///
/// ID of the left tool dock program
///
pub fn subprogram_tool_dock_left() -> SubProgramId { SubProgramId::called("flowbetween::tool_dock::left") }

///
/// ID of the right tool dock program
///
pub fn subprogram_tool_dock_right() -> SubProgramId { SubProgramId::called("flowbetween::tool_dock::right") }

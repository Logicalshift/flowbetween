use flo_scene::*;

///
/// ID of the main document subprogram
///
pub fn subprogram_flowbetween_document() -> SubProgramId { SubProgramId::called("flowbetween::document") }

///
/// ID of the subprogram where DrawingWindowRequests can be sent to
///
pub fn subprogram_window() -> SubProgramId { SubProgramId::called("flowbetween::window") }

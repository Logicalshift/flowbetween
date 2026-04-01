use flo_canvas::*;

///
/// Actions for creating drawing overlays
///
#[derive(Clone, PartialEq, Debug)]
pub enum OverlayAction {
    /// Clears anything in the overlay
    Clear,

    /// Performs drawing actions to the current overlay (appended to the current overlay layer)
    Draw(Vec<Draw>)
}

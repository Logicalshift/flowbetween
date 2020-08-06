///
/// The maximum size of a layout region
///
/// GTK will expand controls to fit their contents: FlowBetween has a layout algorithm based on shrinking controls
/// instead, so we need to ensure when the layout sets a size for a widget that the widget can't grow or this
/// could result in feedback problems.
///
#[derive(Clone, Copy)]
pub struct MaxLayoutSize {
    /// Width of the scroll region, in pixels
    pub width: i32,

    /// Height of the scroll region, in pixels
    pub height: i32
}

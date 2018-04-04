///
/// Describes a region being drawn by a canvas
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CanvasViewport {
    /// The total width of the canvas
    pub width: f32,

    /// The total height of the canvas
    pub height: f32,

    /// The viewport origin X coordinate
    pub viewport_x: f32,

    /// The viewport origin Y coordinate
    pub viewport_y: f32,

    /// The viewport width
    pub viewport_width: f32,

    /// The viewport height
    pub viewport_height: f32
}

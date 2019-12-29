///
/// Describes a region being drawn by a canvas
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CanvasViewport {
    /// The total width of the canvas
    pub width: i32,

    /// The total height of the canvas
    pub height: i32,

    /// The viewport origin X coordinate
    pub viewport_x: i32,

    /// The viewport origin Y coordinate
    pub viewport_y: i32,

    /// The viewport width
    pub viewport_width: i32,

    /// The viewport height
    pub viewport_height: i32
}

impl CanvasViewport {
    pub fn minimal() -> CanvasViewport {
        CanvasViewport {
            width:              1,
            height:             1,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     1,
            viewport_height:    1
        }
    }
}

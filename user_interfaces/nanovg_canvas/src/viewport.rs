use nanovg::*;

///
/// Indicates the viewport that a canvas is intended to represent
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct NanoVgViewport {
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

impl NanoVgViewport {
    pub fn to_transform(&self) -> Transform {
        let scale   = (self.height as f32)/2.0;

        Transform::new()
            .translate(-self.viewport_x as f32, -self.viewport_y as f32)
            .translate((self.width as f32)/2.0, (self.height as f32)/2.0)
            .scale(scale, -scale)
    }
}

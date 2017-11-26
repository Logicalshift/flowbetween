use super::mouse::*;

///
/// The device that caused a painting event
/// 
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum PaintDevice {
    Other,
    Mouse(MouseButton),
    Pen,
    Touch
}

///
/// Possible actions for a paint stroke
///
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum PaintAction {
    /// Start of a paint stroke (mouse/stylus/touch down)
    Start,

    /// Continuation of a paint stroke previously started (drag)
    Continue,

    /// End of a paint stroke (mouse/stylus/touch up)
    Finish
}

///
/// Data for a painting event
///
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub struct Painting {
    /// Action for this painting event
    pub action: PaintAction,

    /// Coordinates relative to the control that was painted upon
    pub location: (f32, f32),

    /// Pressure of this event
    pub pressure: f32,

    /// X tilt (-90 to 90)
    pub tilt_x: f32,

    /// Y tilt (-90 to 90)
    pub tilt_y: f32
}

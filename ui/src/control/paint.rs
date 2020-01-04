use super::mouse::*;

///
/// The device that caused a painting event
///
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum PaintDevice {
    /// Unknown paint device
    Other,

    /// Mouse with a particular button held down
    Mouse(MouseButton),

    /// Pen (with a particular stylus ID in case the user has multiple styluses)
    Pen,

    /// Eraser (with a particular stylus ID in case the user has multiple styluses)
    Eraser,

    /// Finger input on a touch display
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

    /// Predicted continuation of a paint stroke (points added by this method should be removed at the next Continue, which provides the 'real' events)
    Prediction,

    /// End of a paint stroke (mouse/stylus/touch up)
    Finish,

    /// Paint stroke was cancelled
    Cancel
}

///
/// Data for a painting event
///
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub struct Painting {
    /// Action for this painting event
    pub action: PaintAction,

    /// In the event the user has multiple pointers (eg, multiple styluses on a tablet), this is the ID of the stylus that the user is using
    pub pointer_id: i32,

    /// Coordinates relative to the control that was painted upon
    pub location: (f32, f32),

    /// Pressure of this event
    pub pressure: f32,

    /// X tilt (-90 to 90)
    pub tilt_x: f32,

    /// Y tilt (-90 to 90)
    pub tilt_y: f32
}

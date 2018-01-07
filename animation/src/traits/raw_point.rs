///
/// A raw point represents a point from an input device
/// 
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub struct RawPoint {
    /// Where the pointer was on the canvas
    pub position: (f32, f32),

    /// The pressure used
    pub pressure: f32,

    /// The tilt of the device
    pub tilt: (f32, f32)
}

impl From<(f32, f32)> for RawPoint {
    fn from(pos: (f32, f32)) -> RawPoint {
        RawPoint {
            position:   pos,
            tilt:       (0.0, 0.0),
            pressure:   1.0
        }
    }
}

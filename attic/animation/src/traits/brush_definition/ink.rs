
///
/// Ink brushes are solid lines of varying width. This defines how they behave.
/// The actual behaviour is implemented by the `InkBrush` structure.
///
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct InkDefinition {
    /// Width at pressure 0%
    pub min_width: f32,

    /// Width at pressure 100%
    pub max_width: f32,

    // Distance to scale up at the start of the brush stroke
    pub scale_up_distance: f32
}

impl InkDefinition {
    ///
    /// Creates the default ink definition
    ///
    pub fn default() -> InkDefinition {
        InkDefinition {
            min_width:          0.25,
            max_width:          5.0,
            scale_up_distance:  40.0
        }
    }

    ///
    /// Creates the default ink definition for the eraser brush
    ///
    pub fn default_eraser() -> InkDefinition {
        InkDefinition {
            min_width:          3.0,
            max_width:          20.0,
            scale_up_distance:  5.0
        }
    }
}

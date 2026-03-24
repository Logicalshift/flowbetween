use super::brush_point::*;
use crate::scenery::document::canvas::*;

use flo_scene::*;
use serde::*;

use futures::stream::{BoxStream};

use std::sync::*;

///
/// Brushes respond to requests by returning functions that pipe results of one type to another. The brush
/// tool assembles these responses into a pipeline that generates the output shape from the input set of points
///
pub enum BrushResponse {
    /// Filters the input points for the brush stroke (eg, for applying smoothing to the brush stroke, but also for things like
    /// setting the brush radius)
    Points(Arc<dyn Send + Sync + Fn(BoxStream<'static, BrushPoint>) -> BoxStream<'static, BrushPoint>>),

    /// Generates the base shapes that are added to the canvas
    ///
    /// In general, there should only be one of these for a brush. Using more than one will generate multiple
    /// shapes in the canvas.
    ///
    /// The shape generator should accumulate the points (ie, generate shapes by using all the points received so far
    /// plus the new set). Ie, we stream in the points we get from the input and stream out the intermediate shapes that
    /// are shown as the user draws with the brush. This means that the brush can create previews quickly by reusing the
    /// data that it has generated already. (If we're regenerating from a brush stroke, this will receive only a single
    /// set of brush points: we only stream a large set if we're showing an interactive preview to the user)
    ShapeGenerator(Arc<dyn Send + Sync + Fn(BoxStream<'static, Vec<BrushPoint>>) -> BoxStream<'static, ShapeWithProperties>>),

    /// Modifies the shapes: often to apply properties like colour to them
    ///
    /// This is how things like the colour or the line width tool applies its properties to the final brush stroke
    Shapes(Arc<dyn Send + Sync + Fn(BoxStream<'static, Vec<ShapeWithProperties>>) -> BoxStream<'static, Vec<ShapeWithProperties>>>),
}

impl SceneMessage for BrushResponse {
    fn serializable() -> bool { false }
}

impl Serialize for BrushResponse {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::*;
        Err(S::Error::custom("PhysicsLayer cannot be serialized"))
    }
}

impl<'de> Deserialize<'de> for BrushResponse {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::*;
        Err(D::Error::custom("PhysicsLayer cannot be serialized"))
    }
}

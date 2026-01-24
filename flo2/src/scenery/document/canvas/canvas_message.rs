use crate::scenery::ui::*;

use flo_draw::canvas::*;
use flo_scene::*;

use serde::*;
use uuid::*;

///
/// Identifier used for a layer in the canvas document
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasLayerId(Uuid);

impl CanvasLayerId {
    ///
    /// Creates a unique new canvas layer ID
    ///
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

///
/// Identifier used for a path in the canvas document
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasPathId(Uuid);

impl CanvasPathId {
    ///
    /// Creates a unique new canvas path ID
    ///
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

///
/// Basic editing actions for the canvas
///
#[derive(Clone, Serialize, Deserialize)]
pub enum CanvasEdit {
    /// Adds a layer (setting the 'after_layer' to None will create the background layer)
    AddLayer { new_layer_id: CanvasLayerId, after_layer: Option<CanvasLayerId> },

    /// Removes the specified layer
    RemoveLayer(CanvasLayerId),

    /// Adds a path with no properties (transparent fill/stroke) to the canvas
    AddPath(CanvasPathId, UiPath),

    /// Sets the fill colour of a path
    SetFillColor(CanvasPathId, Color),

    /// Sets the stroke colour of a path
    SetStrokeColor(CanvasPathId, Color),

    /// Sets the line width for the outline of a path
    SetLineWidth(CanvasPathId, f64),

    /// Sets the cap and join properties for a path
    SetLineProperties(CanvasPathId, LineCap, LineJoin),
}

impl SceneMessage for CanvasEdit {

}
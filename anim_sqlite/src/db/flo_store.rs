use super::db_enum::*;
use super::flo_query::*;

// TODO: make error type more generic
use rusqlite::*;

use animation::*;
use std::sync::*;
use std::time::Duration;

///
/// Possible updates made to the database. We use a simple stack machine for
/// database updates (so we can re-use IDs).
/// 
/// Items starting 'push' always leave at least one element on the stack.
/// 'Pop' elements remove an element. (Push elements may also remove to use
/// to generate the element they leave behind)
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum DatabaseUpdate {
    /// Removes the ID from the top of the stack
    Pop,

    /// Updates the canvas size of the animation
    UpdateCanvasSize(f64, f64),

    /// Inserts a new edit log and pushes its ID
    PushEditType(EditLogType),

    /// Pops an edit log ID and uses it to create a new set size element
    PopEditLogSetSize(f32, f32),

    /// Sets the layer for the current edit log item, leaving the log ID on the stack
    PushEditLogLayer(u64),

    /// Sets the time of the current edit item, leaving the log ID on the stack
    PushEditLogWhen(Duration),

    /// Pops two things from the stack: a brush ID and an edit ID and inserts a brush with the specified drawing style
    PopEditLogBrush(DrawingStyleType),

    /// Pops two things from the stack: a brush properties ID and an edit ID and inserts a brush properties edit
    PopEditLogBrushProperties,

    /// Uses the edit ID on top of the stack and inserts a raw point for it (index, position, pressure, tilt)
    PushRawPoints(Arc<Vec<RawPoint>>),

    /// Inserts a new brush type definition, pushing the new brush's ID to the stack
    PushBrushType(BrushDefinitionType),

    /// Inserts an ink brush, leaving the brush ID on the stack
    PushInkBrush(f32, f32, f32),

    /// Pops a colour ID and pushes brush properties with that colour and the specified size and opacity
    PushBrushProperties(f32, f32),

    /// Pushes a colour ID of the specified type
    PushColorType(ColorType),

    /// Uses the colour ID on top of the stack and inserts an RGB value, leaving the ID behind
    PushRgb(f32, f32, f32),

    /// Uses the colour ID on top of the stack and inserts an HSLuv value, leaving the ID behind
    PushHsluv(f32, f32, f32),

    /// Removes the layer with the ID on top of the stack
    PopDeleteLayer,

    /// Creates a new layer of the specified type and pushes its ID
    PushLayerType(LayerType),

    /// Takes the layer ID on the stack and
    PushAssignLayer(u64),

    /// Looks up a layer with an assigned ID and pushes its real ID
    PushLayerForAssignedId(u64),

    /// Pushes a known layer ID
    PushLayerId(i64),

    /// Adds a key frame to the layer with the ID on top of the stack
    PopAddKeyFrame(Duration),

    /// Removes a keyframe from the layer with the ID on top of the stack
    PopRemoveKeyFrame(Duration),

    /// Pops a layer ID and pushes the time and ID of the key
    PushNearestKeyFrame(Duration),

    /// Uses the keyframe ID and time on top of the stack and creates a new vector element with the specified type and time (from the start of the animation) and pushes its ID
    /// (Stack has the element ID, the key frame ID and the time left afterwards)
    PushVectorElementType(VectorElementType, Duration),

    /// Pops a brush ID and a vector element ID and creates a vector brush element from them
    PopVectorBrushElement(DrawingStyleType),

    /// Pops a brush properites ID and a vector element ID and creates a vector brush properties element from them
    PopVectorBrushPropertiesElement,

    /// Pops a vector element ID from the stack and creates a set of brush points for it
    PopBrushPoints(Arc<Vec<BrushPoint>>)
}

///
/// Trait implemented by objects that can provide an underlying store for FlowBetween
/// 
pub trait FloStore {
    ///
    /// Performs a set of updates on the store
    /// 
    fn update<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<()>;

    ///
    /// Starts queuing up store updates for later execution as a batch
    /// 
    fn begin_queuing(&mut self);

    ///
    /// Executes the queued events (and stops queueing future events)
    /// 
    fn execute_queue(&mut self) -> Result<()>;

    ///
    /// Ensures any pending updates are committed to the database (but continues to queue future events)
    /// 
    fn flush_pending(&mut self) -> Result<()>;
}

///
/// Trait implemented by objects that can read or write from a Flo data file
/// 
pub trait FloFile : FloStore + FloQuery {

}

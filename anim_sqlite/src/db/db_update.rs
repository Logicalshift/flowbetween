use super::db_enum::*;

use std::time::Duration;

///
/// Possible updates made to the database. We use a simple stack machine for
/// database updates (so we can re-use IDs).
/// 
/// Items starting 'push' always leave at least one element on the stack.
/// 'Pop' elements remove an element. (Push elements may also remove to use
/// to generate the element they leave behind)
/// 
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DatabaseUpdate {
    /// Removes the ID from the top of the stack
    Pop,

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
    PushRawPoint(u64, (f32, f32), f32, (f32, f32)),

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

    /// Creates a new layer of the specified type and pushes its ID
    PushLayerType(LayerType),

    /// Pops a layer ID and inserts an ID assignment for it
    PopAssignLayer(u64),

    /// Adds a key frame to the layer with the specified assigned ID
    AddKeyFrame(u64, Duration),

    /// Removes a keyframe from the layer with the specified assigned ID
    RemoveKeyFrame(u64, Duration),

    /// Looks up a layer with an assigned ID and pushes its real ID
    PushLayerForAssignedId(u64),

    /// Creates a new vector element with the specified type and time (from the start of the animation) and pushes its ID
    PushVectorElementType(VectorElementType, Duration),

    /// Pops a brush ID and a vector element ID and creates a vector brush element from them
    PopVectorBrushElement(DrawingStyleType),

    /// Pops a brush properites ID and a vector element ID and creates a vector brush properties element from them
    PopVectorBrushPropertiesElement,

    /// Given a vector element ID on the stack, creates a brush point (and leaves the element on the stack)
    PushBrushPoint(u64, (f32, f32), (f32, f32), (f32, f32), f32)
}

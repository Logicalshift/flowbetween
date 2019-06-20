use super::db_enum::*;
use super::flo_query::*;
use super::motion_path_type::*;
use super::super::error::*;

use flo_animation::*;
use std::sync::*;
use std::ops::Range;
use std::time::Duration;
use std::result::Result;

///
/// When moving an element relative to itself, determines the direction in which the element should move
///
#[derive(Clone, PartialEq, Debug)]
pub enum DbElementMove {
    ToTop,
    Up,
    ToBottom,
    Down
}

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

    /// Sets the element ID for the current edit log item at the specified index, leaving the log ID on the stack
    PushEditLogElementId(usize, i64),

    /// Sets the time of the current edit item, leaving the log ID on the stack
    PushEditLogWhen(Duration),

    /// Pops two things from the stack: a brush ID and an edit ID and inserts a brush with the specified drawing style
    PopEditLogBrush(DrawingStyleType),

    /// Pops an edit ID and associates a string value with it
    PopEditLogString(u32, String),

    /// Uses the edit ID on top of the stack and sets an integer value (the parameters to this are the value index and the value itself). Edit log items can have arbitrary numbers of such parameters, the index generally counts from 0.
    PushEditLogInt(u32, i64),

    /// Uses the edit ID on top of the stack and sets a floating-point value (the parameters to this are the value index and the value itself)
    PushEditLogFloat(u32, f64),

    /// Pops two things from the stack: a brush properties ID and an edit ID and inserts a brush properties edit
    PopEditLogBrushProperties,

    /// Uses the edit ID on top of the stack and inserts a raw point for it (index, position, pressure, tilt)
    PushRawPoints(Arc<Vec<RawPoint>>),

    /// Uses the edit ID on top of the stack and associates a motion origin with it
    PushEditLogMotionOrigin(f32, f32),

    /// Uses the edit ID on top of the stack and associates a motion type with it
    PushEditLogMotionType(MotionType),

    /// Uses the edit ID on top of the stack and sets the attached element ID
    PushEditLogMotionElement(i64),

    /// Pops the specified number of time point IDs from the stack and creates a motion path from them using the edit ID pushed before them (ie, stack shopuld look like `[edit id, point id, point id, ...]`)
    PushEditLogMotionPath(usize),

    /// Pops a path ID and an edit log ID and associates the path with the edit log ID. Leaves the edit log ID on the stack.
    PushEditLogPath,

    /// Creates a new path from the specified points and pushes the ID
    PushPath(Vec<(f32, f32)>),

    /// Creates a path from the points in the list of path components and pushes the ID
    PushPathComponents(Arc<Vec<PathComponent>>),

    /// Pops a path ID and removes the path points with the specified indexes (moving the other components downwards)
    /// Note: when removing bezier components, they have 3 points: if all three are not removed, then the path will
    /// be invalid.
    PopRemovePathPoints(Range<usize>),

    /// Pops a path ID and inserts a new set of components at the point with the specified index.
    /// Note that bezier components have three points: this must not insert new components between the first
    /// and second control point of a curve or between the second control point and the end point, or the path
    /// will be invalid.
    PopInsertPathComponents(usize, Arc<Vec<PathComponent>>),

    /// Creates a new time point at the specified x, y, time coordinates and pushes its ID to the stack
    PushTimePoint(f32, f32, f32),

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

    /// Takes the layer ID on the stack and sets an assigned ID, leaving the real ID on the stack
    PushAssignLayer(u64),

    /// Looks up a layer with an assigned ID and pushes its real ID
    PushLayerForAssignedId(u64),

    /// Pushes a known layer ID
    PushLayerId(i64),

    /// Pops a layer ID and sets the name to the specified value for that layer
    PopLayerName(String),

    /// Adds a key frame to the layer with the ID on top of the stack
    PopAddKeyFrame(Duration),

    /// Removes a keyframe from the layer with the ID on top of the stack
    PopRemoveKeyFrame(Duration),

    /// Pops a layer ID, and creates (or replaces) the cache data for that layer at the specified time
    PopStoreLayerCache(Duration, CacheType, String),

    /// Pops a layer ID and removes the cache of the specified type from the specified time
    PopDeleteLayerCache(Duration, CacheType),

    /// Pops a layer ID and pushes the time and ID of the key
    PushNearestKeyFrame(Duration),

    /// Creates a new vector element with the specified type, leaving the new element ID on the stack
    PushVectorElementType(VectorElementType),

    /// Pops an element ID, a keyframe ID and a keyframe time and sets the time for the element ID.
    /// This also sets the element Z-order so that it is on top at this time.
    /// Pushes the element ID, key frame ID and time back onto the stack afterwards (element ID on top of the stack)
    PushVectorElementTime(Duration),

    /// Uses the element ID on top of the stack and sets its assigned ID, leaving it on top of the stack
    PushElementAssignId(i64),

    /// Takes an assigned ID and pushes the corresponding element ID
    PushElementIdForAssignedId(i64),

    /// Pops a number of elements from the top of the stack, and one more element ID. Adds the final element as an attachment
    /// to each of the elements popped. Leaves the final element on the stack.
    PushAttachElements(usize),

    /// Pops a number of elements from the top of the stack, and one more element ID. Removes the final element as an attachment
    /// from each of the elements popped. Leaves the final element on the stack.
    PushDetachElements(usize),

    /// Pops the element ID from the top of the stack and pushes the key frame ID and then the element ID
    PushKeyFrameIdForElementId,

    /// Peeks at the element ID from the top of the stack and pushes the path ID associated with it (leaving the stack with the path ID on top, followed by the element ID)
    PushPathIdForElementId,

    /// Pops a brush ID and a vector element ID and creates a vector brush element from them
    PopVectorBrushElement(DrawingStyleType),

    /// Pops a brush properites ID and a vector element ID and creates a vector brush properties element from them
    PopVectorBrushPropertiesElement,

    /// Pops a vector element ID from the stack and creates a set of brush points for it
    PopBrushPoints(Arc<Vec<BrushPoint>>),

    /// Pops an element ID and updates the brush point coordinates for it
    UpdateBrushPointCoords(Arc<Vec<((f32, f32), (f32, f32), (f32, f32))>>),

    /// Pops a path ID and updates the coordinates associated with it
    UpdatePathPointCoords(Arc<Vec<(f32, f32)>>),

    /// Pops a path ID, a brush properties ID, a brush ID and a vector element ID and creates a path element from them
    PopVectorPathElement,

    /// Pops an element ID and a keyframe ID from the stack. Changes the ordering of that element within that keyframe.
    PopVectorElementMove(DbElementMove),

    /// Creates a new motion with the specified ID
    CreateMotion(i64),

    /// Sets the type of a motion
    SetMotionType(i64, MotionType),

    /// Sets the origin of a motion
    SetMotionOrigin(i64, f32, f32),

    /// Pops the specified number of time point IDs from the stack and sets the path of the specified motion to match
    SetMotionPath(i64, MotionPathType, usize),

    /// Removes the motion with the specified ID
    DeleteMotion(i64),
}

///
/// Trait implemented by objects that can provide an underlying store for FlowBetween
/// 
pub trait FloStore {
    ///
    /// Performs a set of updates on the store
    /// 
    fn update<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<(), SqliteAnimationError>;

    ///
    /// Starts queuing up store updates for later execution as a batch
    /// 
    fn begin_queuing(&mut self);

    ///
    /// Executes the queued events (and stops queueing future events)
    /// 
    fn execute_queue(&mut self) -> Result<(), SqliteAnimationError>;

    ///
    /// Ensures any pending updates are committed to the database (but continues to queue future events)
    /// 
    fn flush_pending(&mut self) -> Result<(), SqliteAnimationError>;
}

///
/// Trait implemented by objects that can read or write from a Flo data file
/// 
pub trait FloFile : FloStore + FloQuery {

}

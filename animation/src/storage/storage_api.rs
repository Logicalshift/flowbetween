use std::ops::{Range};
use std::time::{Duration};

///
/// Command that is sent to a storage backend
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StorageCommand {
    /// Writes a serialized version of the file settings
    WriteAnimationProperties(String),

    /// Reads the file settings string
    ReadAnimationProperties,

    /// Appends a serialized edit to the edit log
    WriteEdit(String),

    /// Retrieves the highest unused element ID (this ID and any higher are guaranteed to be unassigned)
    ReadHighestUnusedElementId,

    /// Reads how many edits are currently in the edit log
    ReadEditLogLength,

    /// Reads the edits in a particular range
    ReadEdits(Range<usize>),

    /// Writes the serialized value of an element
    WriteElement(i64, String),

    /// Reads the previously serialized value of an element
    ReadElement(i64),

    /// Removes an element from the storage
    DeleteElement(i64),

    /// Adds a new layer with the specified ID and properties to the storage
    AddLayer(u64, String),

    /// Deletes the layer with a specified ID
    DeleteLayer(u64),

    /// Reads all of the layers stored in this API (as LayerProperties)
    ReadLayers,

    /// Sets the properties for a particular layer
    WriteLayerProperties(u64, String),

    /// Reads the properties for a layer
    ReadLayerProperties(u64),

    /// Adds a key frame to a layer
    AddKeyFrame(u64, Duration),

    /// Removes a key frame from a layer
    DeleteKeyFrame(u64, Duration),

    /// Reads the keyframes that appear in a particular time range for a layer
    ReadKeyFrames(u64, Range<Duration>),

    /// Given a layer ID and an element ID, sets where a particular element appears in that layer
    AttachElementToLayer(u64, i64, Duration),

    /// Removes an element from a layer
    DetachElementFromLayer(i64),

    /// Given an element ID, returns all of the layers and keyframes it's attached to
    ReadElementAttachments(i64),

    /// Returns the elements attached to a particular key frame
    ReadElementsForKeyFrame(u64, Duration),

    /// Writes to the layer cache (parameters are layer id, cache time, key and cache value)
    WriteLayerCache(u64, Duration, String, String),

    /// Removes an entry from the layer cache
    DeleteLayerCache(u64, Duration, String),

    /// Reads from the layer cache (parameters are layer id, cache time and key)
    ReadLayerCache(u64, Duration, String)
}

///
/// Response from a storage backend
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StorageResponse {
    /// The storage was updated
    Updated,

    /// Tried to add an element that already exists
    NotReplacingExisting,

    /// The requested item could not be found
    NotFound,

    /// The number of edits
    NumberOfEdits(usize),

    /// The serialized version of the file properites
    AnimationProperties(String),

    /// The serialized version of the layer properties
    LayerProperties(u64, String),

    /// The highest unused element ID (0 if there are no elements stored yet)
    HighestUnusedElementId(i64),

    /// An edit requested when reading the edit log
    Edit(usize, String),

    /// Start of a read from a keyframe. The two times here are the start and end time of the keyframe from the start of the animation
    KeyFrame(Duration, Duration),

    /// A request was made for a location that is not in a keyframe, but where a keyframe is available at the specified later point
    NotInAFrame(Duration),

    /// The serialized version of the element that was requested
    Element(i64, String),

    /// Returns the (layer, keyframe) pairs that a particular element is attached to
    ElementAttachments(i64, Vec<(u64, Duration)>),

    /// Returns the contents of the requested layer cache
    LayerCache(String),

    /// The storage subsystem encountered an error
    Error(StorageError, String)
}

///
/// Errors from the storage API
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StorageError {
    /// General failure
    General,

    /// The storage could not be initialised
    FailedToInitialise,

    /// The storage cannot continue because of an eariler error
    CannotContinueAfterError
}

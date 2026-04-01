use super::storage_error::*;

use std::time::{Duration};

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

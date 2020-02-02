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
    WriteElement(usize, String),

    /// Reads the previously serialized value of an element
    ReadElement(usize),

    /// Removes an element from the storage
    DeleteElement(usize),

    /// Attaches the second element to the first
    AttachElementToElement(usize, usize),

    /// Reverses the effects of attach element
    DetachElementFromElement(usize, usize),

    /// Adds a new layer with the specified ID to the storage
    AddLayer(usize),

    /// Sets the properties for a particular layer
    WriteLayerProperties(usize, String),

    /// Reads the properties for a layer
    ReadLayerProperties(usize),

    /// Deletes the layer with a specified ID
    DeleteLayer(usize),

    /// Adds a key frame to a layer
    AddKeyFrame(usize, Duration),

    /// Removes a key frame from a layer
    DeleteKeyFrame(usize, Duration),

    /// Given a layer ID and an element ID, sets where a particular element appears in that layer
    AttachElementToLayer(usize, usize, Duration),

    /// Removes an element from a layer
    DetachElementFromLayer(usize),

    /// Returns the elements attached to a particular key frame
    ReadElementsForKeyFrame(usize, Duration)
}

///
/// Response from a storage backend
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StorageResponse {
    /// The storage was updated
    Updated,

    /// The requested item could not be found
    NotFound,

    /// The serialized version of the file properites
    AnimationProperties(String),

    /// The serialized version of the layer properties
    LayerPropeties(usize, String),

    /// The highest unused element ID (0 if there are no elements stored yet)
    HighestUnusedElementId(usize),

    /// The serialized version of the element that was requested
    Element(usize, String)
}

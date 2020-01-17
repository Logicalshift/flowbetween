use super::state::*;
use super::storage_descriptor::*;

///
/// Command that can be issued to a FlowBetween instance
///
#[derive(Clone)]
pub enum FloCommand {
    /// Write out a message describing the version of FlowBetween that this is
    Version,

    /// Requests the current command state
    ReadState,

    /// Sets the state for future commands
    SetState(CommandState),

    /// Sets the input animation
    ReadFrom(StorageDescriptor),

    /// Writes a new animation to the catalog with the specified name
    WriteToCatalog(String),

    /// Moves the current 'write' animation into the 'read' position
    ReadFromWriteAnimation,

    /// Lists the files in the main index
    ListAnimations,

    /// Clears the current set of edits
    ClearEdits,

    /// Reads in all of the edits from the input animation to the edit buffer
    ReadAllEdits,

    /// Writes out a summary of the edits in the edit buffer
    SummarizeEdits,

    /// Serializes the edits to the output
    SerializeEdits,

    /// Writes all of the edits currently in the edit buffer to the output animation
    WriteAllEdits
}

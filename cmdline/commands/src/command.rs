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

    /// Lists the files in the main index
    ListAnimations,

    /// Reads in all of the edits from the input animation to the edit buffer
    ReadAllEdits,

    /// Writes out a summary of the edits in the edit buffer
    SummarizeEdits
}

use super::storage_descriptor::*;

///
/// Command that can be issued to a FlowBetween instance
///
#[derive(Clone)]
pub enum FloCommand {
    /// Write out a message describing the version of FlowBetween that this is
    Version,

    /// Sets the input animation
    ReadFrom(StorageDescriptor),

    /// Sets the output animation
    WriteTo(StorageDescriptor),

    /// Lists the files in the main index
    ListAnimations,

    /// Reads in all of the edits from the input animation to the edit buffer
    ReadAllEdits,

    /// Writes out a summary of the edits in the edit buffer
    SummarizeEdits
}

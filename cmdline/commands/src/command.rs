///
/// Command that can be issued to a FlowBetween instance
///
#[derive(Clone)]
pub enum FloCommand {
    /// Write out a message describing the version of FlowBetween that this is
    Version,

    /// Lists the files in the main index
    ListAnimations
}

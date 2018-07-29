use std::path::PathBuf;

///
/// A file update event from the file manager
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum FileUpdate {
    /// Indicates that a new file has been created at the specified path
    NewFile(PathBuf),

    /// Indicates that the display name for the specified file has changed
    SetDisplayName(PathBuf, String),

    /// The file with the specified path has been removed
    RemovedFile(PathBuf)
}
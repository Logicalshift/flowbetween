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
    RemovedFile(PathBuf),

    /// The file with the specified path has been moved so that it's after the specified file, or at the beginning if that file does not exist
    ChangedOrder(PathBuf, Option<PathBuf>)
}

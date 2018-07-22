///
/// A file update event from the file manager
/// 
pub enum FileUpdate {
    /// Indicates that a new file has been created at the specified path
    NewFile(PathBuf),

    /// The file with the specified path has been removed
    RemovedFile(PathBuf)
}
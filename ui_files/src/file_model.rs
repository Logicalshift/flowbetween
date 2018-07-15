use std::path::Path;

///
/// Trait implemented by model objects that represent open files
/// 
pub trait FileModel : Send+Sync {
    /// The model used for each session editing a single file 
    type InstanceModel;

    ///
    /// Opens the file found at a particular path, returning the model shared across all instances
    /// of this file. This is shared across all controllers using the same file.
    /// 
    fn open(path: &Path) -> Self;

    ///
    /// Creates a new instance model from the shared model. This is used for a single session.
    /// 
    fn new_instance(&self) -> Self::InstanceModel;
}

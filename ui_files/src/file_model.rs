use std::path::Path;
use std::sync::*;

///
/// Trait implemented by model objects that represent open files
///
pub trait FileModel : Send+Sync {
    /// The model used for each session editing a single file
    type InstanceModel;

    /// The type that is used to load files of this type
    type Loader: Send+Sync;

    ///
    /// Opens the file found at a particular path, returning the model shared across all instances
    /// of this file. This is shared across all controllers using the same file.
    ///
    fn open(loader: Arc<Self::Loader>, path: &Path) -> Self;

    ///
    /// Creates a new instance model from the shared model. This is used for a single session.
    ///
    fn new_instance(&self) -> Self::InstanceModel;
}

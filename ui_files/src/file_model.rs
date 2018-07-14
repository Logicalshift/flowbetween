use std::sync::*;
use std::path::Path;

///
/// Trait implemented by model objects that represent open files
/// 
pub trait FileModel {
    /// The part of model that is shared between all open instances of the file
    type SharedModel: Send+Sync;

    /// The model used for each session editing a single file 
    type InstanceModel;

    ///
    /// Opens the file found at a particular path, returning the model shared across all instances
    /// of this file. This is shared across all controllers using the same file.
    /// 
    fn open(path: &Path) -> Self::SharedModel;

    ///
    /// Creates a new instance model from the shared model. This is used for a single session.
    /// 
    fn new_instance(model: Arc<Self::SharedModel>) -> Self::InstanceModel;
}

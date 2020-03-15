use super::flo_model::*;

use flo_animation::*;
use flo_ui_files::*;

use std::sync::*;
use std::path::{Path, PathBuf};

///
/// Represents the file model for FlowBetween animations
///
pub struct FloSharedModel<Loader> {
    /// The loader that can load in a new copy of the animation
    loader: Arc<Loader>,

    /// The path where the animation can be opened from
    path: PathBuf
}

impl<Loader: FileAnimation+Send+Sync+'static> FileModel for FloSharedModel<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
    // TODO: we should probably actually share the file between instances :-)
    type InstanceModel  = FloModel<Loader::NewAnimation>;
    type Loader         = Loader;

    ///
    /// Opens the file found at a particular path, returning the model shared across all instances
    /// of this file. This is shared across all controllers using the same file.
    ///
    fn open(loader: Arc<Loader>, path: &Path) -> FloSharedModel<Loader> {
        FloSharedModel {
            loader: loader,
            path:   PathBuf::from(path),
        }
    }

    ///
    /// Creates a new instance model from the shared model. This is used for a single session.
    ///
    fn new_instance(&self) -> FloModel<Loader::NewAnimation> {
        let animation = self.loader.open(self.path.as_path());

        FloModel::new(animation)
    }
}

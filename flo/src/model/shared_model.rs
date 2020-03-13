use super::flo_model::*;

use flo_animation::*;
use flo_ui_files::*;

use std::marker::PhantomData;
use std::path::{Path, PathBuf};

///
/// Represents the file model for FlowBetween animations
///
pub struct FloSharedModel<Loader> {
    /// The path where the animation can be opened from
    path: PathBuf,

    anim: PhantomData<Loader>
}

impl<Loader: FileAnimation+Send+Sync+'static> FileModel for FloSharedModel<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
    // TODO: we should probably actually share the file between instances :-)
    type InstanceModel = FloModel<Loader::NewAnimation>;

    ///
    /// Opens the file found at a particular path, returning the model shared across all instances
    /// of this file. This is shared across all controllers using the same file.
    ///
    fn open(path: &Path) -> FloSharedModel<Loader> {
        FloSharedModel {
            path: PathBuf::from(path),
            anim: PhantomData
        }
    }

    ///
    /// Creates a new instance model from the shared model. This is used for a single session.
    ///
    fn new_instance(&self) -> FloModel<Loader::NewAnimation> {
        let animation = Loader::open(self.path.as_path());

        FloModel::new(animation)
    }
}

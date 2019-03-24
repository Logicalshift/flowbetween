use super::flo_model::*;

use flo_animation::*;
use flo_ui_files::*;

use std::marker::PhantomData;
use std::path::{Path, PathBuf};

///
/// Represents the file model for FlowBetween animations
/// 
pub struct FloSharedModel<Anim> {
    /// The path where the animation can be opened from
    path: PathBuf,

    anim: PhantomData<Anim>
}

impl<Anim: EditableAnimation+FileAnimation+'static> FileModel for FloSharedModel<Anim> {
    // TODO: we should probably actually share the file between instances :-)
    type InstanceModel = FloModel<Anim>;

    ///
    /// Opens the file found at a particular path, returning the model shared across all instances
    /// of this file. This is shared across all controllers using the same file.
    /// 
    fn open(path: &Path) -> FloSharedModel<Anim> {
        FloSharedModel {
            path: PathBuf::from(path),
            anim: PhantomData
        }
    }

    ///
    /// Creates a new instance model from the shared model. This is used for a single session.
    /// 
    fn new_instance(&self) -> FloModel<Anim> {
        let animation = Anim::open(self.path.as_path());

        FloModel::new(animation)
    }
}

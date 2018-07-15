use super::flo_model::*;

use flo_animation::*;
use flo_ui_files::*;

use std::sync::*;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

///
/// Represents the file model for FlowBetween animations
/// 
pub struct SharedModel<Anim> {
    anim: PhantomData<Anim>
}

impl<Anim: FileAnimation+'static> FileModel for SharedModel<Anim> {
    // TODO: we should probably actually share the file between instances :-)
    type SharedModel = (PathBuf);
    type InstanceModel = FloModel<Anim>;

    ///
    /// Opens the file found at a particular path, returning the model shared across all instances
    /// of this file. This is shared across all controllers using the same file.
    /// 
    fn open(path: &Path) -> (PathBuf) {
        (PathBuf::from(path))
    }

    ///
    /// Creates a new instance model from the shared model. This is used for a single session.
    /// 
    fn new_instance(model: Arc<(PathBuf)>) -> FloModel<Anim> {
        let animation = Anim::open(model.as_path());

        FloModel::new(animation)
    }
}

use crate::sidebar::*;
use crate::model::flo_model::*;

use flo_animation::*;
use flo_animation::undo::*;
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

impl<Loader: 'static+FileAnimation+Send+Sync> FileModel for FloSharedModel<Loader>
where Loader::NewAnimation: 'static+Unpin+EditableAnimation {
    // TODO: we should probably actually share the file between instances :-)
    type InstanceModel  = FloModel<UndoableAnimation<Loader::NewAnimation>>;
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
    fn new_instance(&self) -> FloModel<UndoableAnimation<Loader::NewAnimation>> {
        // Create the animation
        let animation   = self.loader.open(self.path.as_path());
        let animation   = UndoableAnimation::new(animation);

        // The base model
        let model       = FloModel::new(animation);

        // Apply some extra default behaviours
        let model_arc   = Arc::new(model.clone());
        model.sidebar().set_document_panels(document_panels(&model_arc));
        model.sidebar().set_selection_panels(selection_panels(&model_arc));

        model
    }
}

use super::super::file_model::*;

use flo_ui::*;

///
/// A file controller is a controller that can manage a type of file specified by a file model
///
pub trait FileController : Controller+PartialEq {
    /// The model that this controller needs to be constructed
    type Model: FileModel;

    ///
    /// Creates this controller with the specified instance model
    ///
    fn open(model: <Self::Model as FileModel>::InstanceModel) -> Self;
}

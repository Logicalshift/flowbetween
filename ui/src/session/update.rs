use super::super::control::*;
use super::super::viewmodel_update::*;

use flo_canvas::*;

///
/// Represents a differnce to the UI
///
#[derive(Clone, PartialEq, Debug)]
pub struct UiDiff {
    /// The address of where the UI tree was changed
    pub address: Vec<u32>,

    // The UI that replaces the tree at this address
    pub new_ui: Control
}

///
/// Represents a difference to a canvas
///
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct CanvasDiff {
    /// The path to the controller that owns the canvas being updated
    pub controller: Vec<String>,

    /// The name of the canvas being updated
    pub canvas_name: String,

    // The updates being made to the canvas
    pub updates: Vec<Draw>
}

///
/// Updates that can arrive from the UI
///
#[derive(Clone, PartialEq, Debug)]
pub enum UiUpdate {
    /// Start of a stream of UI updates
    Start,

    /// Represents a series of updates to the UI tree
    UpdateUi(Vec<UiDiff>),

    /// Represents an update to a canvas in a controller
    UpdateCanvas(Vec<CanvasDiff>),

    /// Represents an update to the viewmodel
    UpdateViewModel(Vec<ViewModelUpdate>)
}

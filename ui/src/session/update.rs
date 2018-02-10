use super::super::control::*;

use canvas::*;

///
/// Represents a differnce to the UI
/// 
#[derive(Clone, PartialEq)]
pub struct UiDiff {
    /// The address of where the UI tree was changed
    pub address: Vec<u32>,

    // The UI that replaces the tree at this address
    pub new_ui: Control
}

///
/// Represents a difference to a canvas
/// 
#[derive(Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, PartialEq)]
pub enum UiUpdate {
    /// Represents a series of updates to the UI tree
    UpdateUi(Vec<UiDiff>),

    /// Represents an update to a canvas in a controller
    UpdateCanvas(Vec<CanvasDiff>)
}

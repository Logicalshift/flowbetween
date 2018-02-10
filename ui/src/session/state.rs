use super::canvas_state::*;
use super::super::control::*;
use super::super::diff_viewmodel::*;

///
/// Represents the most recent state of a UI session
/// 
pub struct UiSessionState {
    /// The control state at the last update
    ui: Option<Control>,

    /// Creates watchers for the viewmodel
    view_model_diff: Option<DiffViewModel>,

    /// The view model watcher
    view_model_watcher: Option<WatchViewModel>,

    /// The canvas state object
    canvas_state: Option<CanvasState>
}

impl UiSessionState {
    ///
    /// Creates a new UI session state (which initially stores no state)
    /// 
    pub fn new() -> UiSessionState {
        UiSessionState {
            ui:                 None,
            view_model_diff:    None,
            view_model_watcher: None,
            canvas_state:       None
        }
    }
}
use std::sync::*;

///
/// The state of a view in the Cocoa UI
///
pub struct ViewState {
    /// The identifier that has been assigned to this view
    view_id: usize,

    /// The name of the controller that this view belongs to
    controller: Option<Arc<String>>,

    /// The child views for this view
    child_views: Vec<ViewState>
}

impl ViewState {
    ///
    /// Creates a new view state
    ///
    pub fn new(view_id: usize) -> ViewState {
        ViewState {
            view_id:        view_id,
            controller:     None,
            child_views:    vec![]
        }
    }
}

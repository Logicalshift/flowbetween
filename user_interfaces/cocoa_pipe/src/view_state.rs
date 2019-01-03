use super::action::*;
use super::view_type::*;

use flo_ui::*;

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

    ///
    /// Retrieves the child state at the specified address
    ///
    pub fn get_state_at_address(&self, address: &Vec<u32>) -> Option<&ViewState> {
        // The empty address is this view state
        let mut view = self;

        // Follow the address to find the view
        for child_index in address.iter() {
            let child_index = *child_index as usize;

            if child_index < view.child_views.len() {
                view = &view.child_views[child_index];
            } else {
                return None;
            }
        }

        Some(view)
    }

    ///
    /// Retrieves the ID of this view state
    ///
    pub fn id(&self) -> usize { self.view_id }

    ///
    /// Adds the state of a subview to this state
    ///
    pub fn add_child_state(&mut self, new_state: ViewState) {
        self.child_views.push(new_state);
    }

    ///
    /// Sets up this state from a control, and returns the action steps needed to initialise it
    ///
    pub fn set_up_from_control(&mut self, control: &Control) -> Vec<AppAction> {
        // Create the list of set up steps
        let mut set_up_steps = vec![];

        // Create the view with the appropriate type
        set_up_steps.push(AppAction::CreateView(self.view_id, ViewType::from(control)));

        set_up_steps
    }
}

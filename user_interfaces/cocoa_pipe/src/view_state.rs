use super::action::*;
use super::view_type::*;
use super::actions_from::*;

use flo_ui::*;
use flo_canvas::*;

use std::sync::*;

///
/// The state of a view in the Cocoa UI
///
pub struct ViewState {
    /// The identifier that has been assigned to this view
    view_id: usize,

    /// The name of the controller that should be applied to subcontrollers
    subview_controller: Option<Arc<String>>,

    /// The child views for this view
    child_views: Vec<ViewState>,

    /// The canvas for this view, if it has one
    canvas: Option<Canvas>
}

impl ViewState {
    ///
    /// Creates a new view state
    ///
    pub fn new(view_id: usize) -> ViewState {
        ViewState {
            view_id:            view_id,
            subview_controller: None,
            child_views:        vec![],
            canvas:             None
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
    /// If the subviews of this view have a different controller, this will return it
    ///
    pub fn get_subview_controller(&self) -> Option<Arc<String>> {
        self.subview_controller.clone()
    }

    ///
    /// Retrieves the controller path for a given address relative to this view model
    ///
    pub fn get_controller_path_at_address(&self, address: &Vec<u32>) -> Vec<Arc<String>> {
        let mut controller_path = vec![];
        let mut view            = self;

        // Follow the address to find the controller path
        for child_index in address.iter() {
            // If the subviews of this control are under a different controller, then add it to the existing path
            if let Some(controller) = &view.subview_controller {
                controller_path.push(Arc::clone(controller));
            }

            // Move to the subview
            let child_index = *child_index as usize;

            if child_index < view.child_views.len() {
                view = &view.child_views[child_index];
            } else {
                break;
            }
        }

        controller_path
    }

    ///
    /// Retrieves the ID of this view state
    ///
    pub fn id(&self) -> usize { self.view_id }

    ///
    /// Retrieves an iterator over the child views of this view
    ///
    pub fn subviews(&self) -> impl Iterator<Item=&ViewState> { self.child_views.iter() }

    ///
    /// Adds the state of a subview to this state
    ///
    pub fn add_child_state(&mut self, new_state: ViewState) {
        self.child_views.push(new_state);
    }

    ///
    /// Replaces the view at the specified address with a new view
    ///
    pub fn replace_child_state(&mut self, address: &Vec<u32>, new_state: ViewState) {
        // The empty address is this view state
        let mut view = self;

        // Follow the address to find the view
        for child_index in address.iter() {
            let child_index = *child_index as usize;

            if child_index < view.child_views.len() {
                view = &mut view.child_views[child_index];
            } else {
                return;
            }
        }

        // Replace the content of this child view
        *view = new_state;
    }

    ///
    /// Returns the actions required to remove the tree of views starting at this one
    ///
    pub fn destroy_subtree_actions(&self) -> Vec<AppAction> {
        vec![
            AppAction::View(self.view_id, ViewAction::RemoveFromSuperview),
            AppAction::DeleteView(self.view_id)
        ].into_iter()
        .chain(self.child_views.iter()
            .flat_map(|child_view| child_view.destroy_subtree_actions())
            .chain(vec![
                AppAction::View(self.view_id, ViewAction::RemoveFromSuperview),
                AppAction::DeleteView(self.view_id)
            ]))
        .collect()
    }

    ///
    /// Sets up this state from a control, and returns the action steps needed to initialise it
    ///
    pub fn set_up_from_control<BindProperty: FnMut(Property) -> AppProperty>(&mut self, control: &Control, mut bind_property: BindProperty) -> Vec<AppAction> {
        // Create the list of set up steps
        let mut set_up_steps = vec![];

        // Create the view with the appropriate type
        set_up_steps.push(AppAction::CreateView(self.view_id, ViewType::from(control)));

        // Specify the controller name, if there is one
        if let Some(controller_name) = control.controller() {
            self.subview_controller = Some(Arc::new(String::from(controller_name)));
        }

        // Set up the view from its attributes
        let view_set_up = control.attributes()
            .flat_map(move |attribute| attribute.actions_from(&mut bind_property))
            .map(|view_action| AppAction::View(self.view_id, view_action));
        set_up_steps.extend(view_set_up);

        set_up_steps
    }
}

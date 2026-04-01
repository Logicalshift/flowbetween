use super::action::*;
use super::view_type::*;
use super::actions_from::*;

use flo_ui::*;

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

    /// The name of the canvas this view is using, if it has one
    canvas_name: Option<String>
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
            canvas_name:        None
        }
    }

    ///
    /// Sets the canvas name for this view
    ///
    pub fn set_canvas_name(&mut self, name: String) {
        self.canvas_name = Some(name);
    }

    ///
    /// Retrieves the canvas name for this view
    ///
    pub fn canvas_name(&self) -> Option<&String> {
        self.canvas_name.as_ref()
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
    /// Returns true if the subcomponents of a control are just a label
    ///
    fn subcomponents_is_just_label(control: &Control) -> bool {
        if let Some(subcomponents) = control.subcomponents() {
            if subcomponents.len() == 1 {
                let control_type = subcomponents[0].control_type();

                if subcomponents[0].image_resource().is_some() {
                    // Image items aren't labels
                    false
                } else if subcomponents[0].canvas_resource().is_some() {
                    // Canvas items aren't labels
                    false
                } else if control_type == ControlType::Label {
                    // One subcomponent, which is a label
                    true
                } else if control_type == ControlType::Empty {
                    if subcomponents[0].subcomponents().map(|components| components.len()).unwrap_or(0) == 0 {
                        // One subcomponent, which is empty
                        true
                    } else {
                        // One subcomponent, which contains other components
                        false
                    }
                } else {
                    // One subcomponent, which is of some type other than label
                    false
                }
            } else {
                // More than one subcomponent
                false
            }
        } else {
            // No subcomponents
            true
        }
    }

    ///
    /// Returns the steps required for setting up a button
    ///
    /// A button that has no subcomponents or just a label can be set up as a button control.
    /// Cocoa's standard button cannot have sub-views or be larger than a certain size, so other buttons will
    /// be set up as a container button.
    ///
    fn set_up_from_button<BindProperty: FnMut(Property) -> AppProperty>(&mut self, control: &Control, container_has_class: bool, bind_property: &mut BindProperty) -> Vec<AppAction> {
        // Get the properties we use to make a decision
        let has_class           = container_has_class || control.attributes().any(|attr| match attr { ControlAttribute::HintAttr(Hint::Class(_)) => true, _ => false });
        let has_subcomponents   = control.subcomponents().is_some();
        let has_image           = control.image_resource().is_some() || control.canvas_resource().is_some();

        if has_subcomponents && !has_class && !has_image {
            // Can use a standard button if the only subcomponent is a label
            if Self::subcomponents_is_just_label(control) {
                // Use a standard button as these can display labels
                let setup_actions = self.set_up_from_generic_control(control, ViewType::Button, bind_property);

                setup_actions
            } else {
                // Not just a label: need to use the container button
                self.set_up_from_generic_control(control, ViewType::ContainerButton, bind_property)
            }
        } else if !has_class && !has_image {
            // Can use a standard button
            self.set_up_from_generic_control(control, ViewType::Button, bind_property)
        } else {
            // Need to use a container button
            self.set_up_from_generic_control(control, ViewType::ContainerButton, bind_property)
        }
    }

    ///
    /// Perform the 'standard' set of set-up actions
    ///
    fn set_up_from_generic_control<BindProperty: FnMut(Property) -> AppProperty>(&mut self, control: &Control, view_type: ViewType, bind_property: &mut BindProperty) -> Vec<AppAction> {
        // Create the list of set up steps
        let mut set_up_steps = vec![];

        // Create the view with the appropriate type
        set_up_steps.push(AppAction::CreateView(self.view_id, view_type));

        // Specify the controller name, if there is one
        if let Some(controller_name) = control.controller() {
            self.subview_controller = Some(Arc::new(String::from(controller_name)));
        }

        // Set up the view from its attributes (except for the canvas, which is done last)
        let view_set_up = control.attributes()
            .filter(|attribute| if let ControlAttribute::Canvas(_) = attribute { false } else { true })
            .flat_map(|attribute| attribute.actions_from(bind_property))
            .map(|view_action| AppAction::View(self.view_id, view_action));
        set_up_steps.extend(view_set_up);

        // Check for the fast drawing hint
        let fast_drawing = control.attributes().any(|attr| if let ControlAttribute::HintAttr(Hint::FastDrawing) = attr { true } else { false } );

        // Perform drawing/etc actions after the size has been set up
        let canvas_set_up = control.attributes()
            .filter_map(|attribute| if let ControlAttribute::Canvas(canvas) = attribute { Some(canvas) } else { None })
            .map(|canvas| {
                let drawing = canvas.get_drawing();
                if fast_drawing {
                    ViewAction::DrawGpu(drawing)
                } else {
                    ViewAction::Draw(drawing)
                }
            }).map(|view_action| AppAction::View(self.view_id, view_action));
        set_up_steps.extend(canvas_set_up);

        set_up_steps
    }

    ///
    /// Sets up this state from a control, and returns the action steps needed to initialise it
    ///
    /// The 'container_has_class' flag is used to signal whether or not the container for this particular control
    /// has been assigned a class (this generally indicates that we shouldn't use the built-in OS X controls as
    /// they don't support many styles)
    ///
    pub fn set_up_from_control<BindProperty: FnMut(Property) -> AppProperty>(&mut self, control: &Control, container_has_class: bool, mut bind_property: BindProperty) -> Vec<AppAction> {
        let control_type = control.control_type();

        // Some controls need different actions depending on how they're set up
        match control_type {
            // Buttons might be simple OS X button controls or 'container buttons' that have other views on them
            ControlType::Button => self.set_up_from_button(control, container_has_class, &mut bind_property),

            // Other controls get the standard set up
            _                   => self.set_up_from_generic_control(control, ViewType::from(control), &mut bind_property)
        }
    }
}

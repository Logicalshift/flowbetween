use super::types::*;
use super::bounds::*;
use super::modifier::*;
use super::attributes::*;

use super::super::image;
use super::super::diff::*;
use super::super::property::*;
use super::super::binding_canvas::*;
use super::super::resource_manager::*;

use ControlType::*;
use ControlAttribute::*;

///
/// Represents a control
///
#[derive(Clone, PartialEq, Debug)]
pub struct Control {
    /// Attributes for this control
    attributes: Vec<ControlAttribute>,

    /// Type of this control
    control_type: ControlType
}

impl Control {
    /// Creates a new control of a particular type
    pub fn new(control_type: ControlType) -> Control {
        Control { attributes: vec![], control_type: control_type }
    }

    /// Creates a new container control
    pub fn container() -> Control {
        Self::new(Container)
    }

    /// Creates a new cropping container control
    pub fn cropping_container() -> Control {
        Self::new(CroppingContainer)
    }

    /// Creates a new scrolling container control
    pub fn scrolling_container() -> Control {
        Self::new(ScrollingContainer)
    }

    /// Creates a new popup control
    pub fn popup() -> Control {
        Self::new(Popup)
    }

    /// Creates a new button control
    pub fn button() -> Control {
        Self::new(Button)
    }

    /// Creates a new label control
    pub fn label() -> Control {
        Self::new(Label)
    }

    /// Creates a new text box control
    pub fn text_box() -> Control {
        Self::new(TextBox)
    }

    /// Creates a new checkbox control
    pub fn check_box() -> Control {
        Self::new(CheckBox)
    }

    /// Create a new empty control
    pub fn empty() -> Control {
        Self::new(Empty)
    }

    /// Creates a new canvas control
    pub fn canvas() -> Control {
        Self::new(ControlType::Canvas)
    }

    /// Creates a new slider control
    pub fn slider() -> Control {
        Self::new(ControlType::Slider)
    }

    /// Creates a new rotor control
    pub fn rotor() -> Control {
        Self::new(ControlType::Rotor)
    }

    /// Adds an attribute to this control
    pub fn add_attribute(&mut self, attribute: ControlAttribute) {
        self.attributes.push(attribute);
    }

    /// Creates a control with some attributes added to it
    pub fn with<T: ControlModifier>(mut self, modifier: T) -> Control {
        modifier.modify(&mut self);

        self
    }

    ///
    /// Creates a control with an added controller
    ///
    pub fn with_controller(self, controller: &str) -> Control {
        self.with(ControlAttribute::Controller(String::from(controller)))
    }

    /// Returns an iterator over the attributes for this control
    pub fn attributes<'a>(&'a self) -> Box<dyn Iterator<Item=&'a ControlAttribute>+'a> {
        Box::new(self.attributes.iter())
    }

    /// The type of this control
    pub fn control_type(&self) -> ControlType {
        self.control_type
    }

    ///
    /// True if any of the attributes of this control exactly match the specified attribute
    /// (using the rules of is_different_flat, so no recursion when there are subcomponents)
    ///
    pub fn has_attribute_flat(&self, attr: &ControlAttribute) -> bool {
        self.attributes.iter()
            .any(|test_attr| !test_attr.is_different_flat(attr))
    }

    ///
    /// If this control has a controller attribute, finds it
    ///
    pub fn controller<'a>(&'a self) -> Option<&'a str> {
        self.attributes.iter()
            .map(|attr| attr.controller())
            .find(|attr| attr.is_some())
            .map(|attr| attr.unwrap())
    }

    ///
    /// If this control has subcomponents, finds them
    ///
    pub fn subcomponents<'a>(&'a self) -> Option<&'a Vec<Control>> {
        self.attributes.iter()
            .map(|attr| attr.subcomponents())
            .find(|attr| attr.is_some())
            .map(|attr| attr.unwrap())
    }

    ///
    /// Finds the bounding box for this control if it exists
    ///
    pub fn bounding_box<'a>(&'a self) -> Option<&'a Bounds> {
        self.attributes.iter()
            .map(|attr| attr.bounding_box())
            .find(|attr| attr.is_some())
            .map(|attr| attr.unwrap())
    }

    ///
    /// Finds the text for this control if it exists
    ///
    pub fn text<'a>(&'a self) -> Option<&'a Property> {
        self.attributes.iter()
            .map(|attr| attr.text())
            .find(|attr| attr.is_some())
            .map(|attr| attr.unwrap())
    }

    ///
    /// If this control has a canvas attribute, finds it
    ///
    pub fn canvas_resource<'a>(&'a self) -> Option<&Resource<BindingCanvas>> {
        self.attributes.iter()
            .map(|attr| attr.canvas())
            .find(|attr| attr.is_some())
            .map(|attr| attr.unwrap())
    }

    ///
    /// If this control has a canvas attribute, finds it
    ///
    pub fn image_resource<'a>(&'a self) -> Option<&Resource<image::Image>> {
        self.attributes.iter()
            .map(|attr| attr.image())
            .find(|attr| attr.is_some())
            .map(|attr| attr.unwrap())
    }

    ///
    /// Finds the names of all of the controllers referenced by this control and its subcontrols
    ///
    pub fn all_controllers(&self) -> Vec<String> {
        let mut result = vec![];

        fn all_controllers(ctrl: &Control, result: &mut Vec<String>) {
            // Push the controller to the result if there is one
            if let Some(controller_name) = ctrl.controller() {
                result.push(String::from(controller_name));
            }

            // Go through the subcomponents as well
            if let Some(subcomponents) = ctrl.subcomponents() {
                for subcomponent in subcomponents.iter() {
                    all_controllers(subcomponent, result);
                }
            }
        }

        all_controllers(self, &mut result);

        // Remove duplicate controllers
        result.sort();
        result.dedup();

        result
    }

    ///
    /// Visits the control tree and performs a mapping function on each item
    ///
    pub fn map<TFn: Fn(&Control) -> Control>(&self, map_fn: &TFn) -> Control {
        // Map this control
        let mut new_control = map_fn(self);

        // Map any subcomponents that might exist
        let num_attributes = new_control.attributes.len();
        for index in 0..num_attributes {
            // TODO: we really only want to update the attribute if
            // it's a subcomponents attribute but we end up with an
            // awkward code structure as there's no elegant way to
            // release the borrow caused by the subcomponents ref in
            // the if statement here before updating the value. This
            // construction looks better but clones all the attributes
            // to leave them unupdated
            new_control.attributes[index] =
                if let SubComponents(ref subcomponents) = new_control.attributes[index] {
                    // Map each of the subcomponents
                    let mut new_subcomponents = vec![];

                    for component in subcomponents.iter() {
                        new_subcomponents.push(component.map(map_fn));
                    }

                    ControlAttribute::SubComponents(new_subcomponents)
                } else {
                    // Attribute remains the same
                    new_control.attributes[index].clone()
                };
        }

        new_control
    }
}

impl DiffableTree for Control {
    fn child_nodes<'a>(&'a self) -> Vec<&'a Self> {
        self.attributes
            .iter()
            .map(|attr| attr.subcomponents().map(|component| component.iter()))
            .filter(|maybe_components| maybe_components.is_some())
            .flat_map(|components| components.unwrap())
            .collect()
    }

    fn is_different(&self, compare_to: &Self) -> bool {
        self.control_type() != compare_to.control_type()
            || self.attributes.iter().any(|attr| !compare_to.has_attribute_flat(attr))
    }
}

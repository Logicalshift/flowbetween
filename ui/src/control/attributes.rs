use super::bounds::*;
use super::control::*;
use super::actions::*;

use super::super::image;
use super::super::canvas;
use super::super::property::*;
use super::super::resource_manager::*;
 
///
/// Attribute attached to a control
///
#[derive(Clone, PartialEq)]
pub enum ControlAttribute {
    /// The bounding box for this control
    BoundingBox(Bounds),

    /// The text for this control
    Text(Property),

    /// Whether or not this control is selected
    Selected(Property),

    /// The unique ID for this control
    Id(String),

    /// Subcomponents of this control
    SubComponents(Vec<Control>),

    /// Specifies the controller that manages the subcomponents of this control
    Controller(String),

    /// When the specified action occurs for this item, send the event 
    /// denoted by the string to the controller
    Action(ActionTrigger, String),

    /// Specifies the background image for this control
    Image(Resource<image::Image>),

    /// Specifies the canvas to use for this control (assuming it's a canvas control)
    Canvas(Resource<canvas::Canvas>)
}

impl ControlAttribute {
    ///
    /// The bounding box represented by this attribute
    ///
    pub fn bounding_box<'a>(&'a self) -> Option<&'a Bounds> {
        match self {
            &BoundingBox(ref bounds)    => Some(bounds),
            _                           => None
        }
    }

    ///
    /// The text represented by this attribute
    ///
    pub fn text<'a>(&'a self) -> Option<&'a Property> {
        match self {
            &Text(ref text) => Some(text),
            _               => None
        }
    }

    ///
    /// The ID represented by this attribute
    ///
    pub fn id<'a>(&'a self) -> Option<&'a String> {
        match self {
            &Id(ref id) => Some(id),
            _           => None
        }
    }

    ///
    /// The subcomponent represented by this attribute
    ///
    pub fn subcomponents<'a>(&'a self) -> Option<&'a Vec<Control>> {
        match self {
            &SubComponents(ref components)  => Some(components),
            _                               => None
        }
    }

    ///
    /// The controller represented by this attribute
    ///
    pub fn controller<'a>(&'a self) -> Option<&'a str> {
        match self {
            &Controller(ref controller) => Some(controller),
            _                           => None
        }
    }

    ///
    /// The action represented by this attribute
    ///
    pub fn action<'a>(&'a self) -> Option<(&'a ActionTrigger, &'a String)> {
        match self {
            &Action(ref trigger, ref action)    => Some((trigger, action)),
            _                                   => None
        }
    }

    ///
    /// Property representing whether or not this control is selected
    ///
    pub fn selected<'a>(&'a self) -> Option<&'a Property> {
        match self {
            &Selected(ref is_selected)  => Some(is_selected),
            _                           => None
        }
    }

    ///
    /// The image resource for this control, if there is one
    ///
    pub fn image<'a>(&'a self) -> Option<&'a Resource<image::Image>> {
        match self {
            &Image(ref image)   => Some(image),
            _                   => None
        }
    }

    ///
    /// The canvas resource for this control, if there is one
    ///
    pub fn canvas<'a>(&'a self) -> Option<&'a Resource<canvas::Canvas>> {
        match self {
            &Canvas(ref canvas) => Some(canvas),
            _                   => None
        }
    }

    ///
    /// Returns true if this attribute is different from another one
    /// (non-recursively, so this won't check subcomoponents)
    ///
    pub fn is_different_flat(&self, compare_to: &ControlAttribute) -> bool {
        match self {
            &BoundingBox(ref bounds)            => Some(bounds) != compare_to.bounding_box(),
            &Text(ref text)                     => Some(text) != compare_to.text(),
            &Id(ref id)                         => Some(id) != compare_to.id(),
            &Controller(ref controller)         => Some(controller.as_ref()) != compare_to.controller(),
            &Action(ref trigger, ref action)    => Some((trigger, action)) != compare_to.action(),
            &Selected(ref is_selected)          => Some(is_selected) != compare_to.selected(),
            &Image(ref image_resource)          => Some(image_resource) != compare_to.image(),
            &Canvas(ref canvas_resource)        => Some(canvas_resource) != compare_to.canvas(),

            // For the subcomponents we only care about the number as we don't want to recurse
            &SubComponents(ref components)      => Some(components.len()) != compare_to.subcomponents().map(|components| components.len())
        }
    }
}

use ControlAttribute::*;

///
/// Trait implemented by things that can be converted into control attributes
///
pub trait ToControlAttributes {
    fn attributes(&self) -> Vec<ControlAttribute>;
}

impl ToControlAttributes for ControlAttribute {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![self.clone()]
    }
}

impl<'a> ToControlAttributes for &'a str {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![Text(self.to_property())]
    }
}

impl ToControlAttributes for Bounds {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![BoundingBox(self.clone())]
    }
}

impl ToControlAttributes for Resource<image::Image> {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![Image(self.clone())]
    }
}

impl ToControlAttributes for Resource<canvas::Canvas> {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![ControlAttribute::Canvas(self.clone())]
    }
}

impl ToControlAttributes for (ActionTrigger, String) {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![Action(self.0.clone(), self.1.clone())]
    }
}

impl ToControlAttributes for Vec<ControlAttribute> {
    fn attributes(&self) -> Vec<ControlAttribute> {
        self.clone()
    }
}

impl<A: ToControlAttributes, B: ToControlAttributes> ToControlAttributes for (A, B) {
    fn attributes(&self) -> Vec<ControlAttribute> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());

        res
    }
}

impl<A: ToControlAttributes, B: ToControlAttributes, C: ToControlAttributes> ToControlAttributes for (A, B, C) {
    fn attributes(&self) -> Vec<ControlAttribute> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());
        res.append(&mut self.2.attributes());

        res
    }
}

impl<A: ToControlAttributes, B: ToControlAttributes, C: ToControlAttributes, D: ToControlAttributes> ToControlAttributes for (A, B, C, D) {
    fn attributes(&self) -> Vec<ControlAttribute> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());
        res.append(&mut self.2.attributes());
        res.append(&mut self.3.attributes());

        res
    }
}

impl ToControlAttributes for Vec<Control> {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![SubComponents(self.iter().cloned().collect())]
    }
}

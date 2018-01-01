use super::bounds::*;
use super::control::*;
use super::actions::*;
use super::font_attr::*;
use super::appearance_attr::*;

use super::super::property::*;
use super::super::binding_canvas::*;
use super::super::resource_manager::*;

use modifier::*;
 
///
/// Attribute attached to a control
///
#[derive(Clone, PartialEq, Debug)]
pub enum ControlAttribute {
    /// The bounding box for this control
    BoundingBox(Bounds),

    /// The text for this control
    Text(Property),

    /// Specifies the font properties of this control
    FontAttribute(FontAttr),

    /// Specifies the appearance of this control
    AppearanceAttr(Appearance),

    // TODO: state attribute
    /// Whether or not this control is selected
    Selected(Property),

    /// Whether or not this control has a badge attached to it
    Badged(Property),

    /// The unique ID for this control
    Id(String),

    /// Subcomponents of this control
    SubComponents(Vec<Control>),

    /// Specifies the controller that manages the subcomponents of this control
    Controller(String),

    /// When the specified action occurs for this item, send the event 
    /// denoted by the string to the controller
    Action(ActionTrigger, String),

    // TODO: content attribute (maybe with text?). Image might be appearance though
    /// Specifies the canvas to use for this control (assuming it's a canvas control)
    Canvas(Resource<BindingCanvas>),
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
    /// The font atrributes represented by this attribute
    /// 
    pub fn font<'a>(&'a self) -> Option<&'a FontAttr> {
        match self {
            &FontAttribute(ref attr)    => Some(attr),
            _                           => None
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
    /// Property representing whether or not this control has a badge
    ///
    pub fn badged<'a>(&'a self) -> Option<&'a Property> {
        match self {
            &Badged(ref is_badged)  => Some(is_badged),
            _                       => None
        }
    }

    ///
    /// The canvas resource for this control, if there is one
    ///
    pub fn canvas<'a>(&'a self) -> Option<&'a Resource<BindingCanvas>> {
        match self {
            &Canvas(ref canvas) => Some(canvas),
            _                   => None
        }
    }
    
    ///
    /// The appearance assigned by this attribute, if there is one
    /// 
    pub fn appearance<'a>(&'a self) -> Option<&'a Appearance> {
        match self {
            &AppearanceAttr(ref appearance) => Some(appearance),
            _                               => None
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
            &FontAttribute(ref font)            => Some(font) != compare_to.font(),
            &Id(ref id)                         => Some(id) != compare_to.id(),
            &Controller(ref controller)         => Some(controller.as_ref()) != compare_to.controller(),
            &Action(ref trigger, ref action)    => Some((trigger, action)) != compare_to.action(),
            &Selected(ref is_selected)          => Some(is_selected) != compare_to.selected(),
            &Badged(ref is_badged)              => Some(is_badged) != compare_to.badged(),
            &Canvas(ref canvas_resource)        => Some(canvas_resource) != compare_to.canvas(),
            &AppearanceAttr(ref appearance)     => Some(appearance) != compare_to.appearance(),

            // For the subcomponents we only care about the number as we don't want to recurse
            &SubComponents(ref components)      => Some(components.len()) != compare_to.subcomponents().map(|components| components.len())
        }
    }
}

use ControlAttribute::*;

impl<'a> Modifier<Control> for &'a str {
    fn modify(self, control: &mut Control) {
        control.add_attribute(Text(self.to_property()))
    }
}

impl Modifier<Control> for String {
    fn modify(self, control: &mut Control) {
        control.add_attribute(Text(self.to_property()))
    }
}

impl<'a> Modifier<Control> for &'a String {
    fn modify(self, control: &mut Control) {
        control.add_attribute(Text(self.to_property()))
    }
}
impl Modifier<Control> for Bounds {
    fn modify(self, control: &mut Control) {
        control.add_attribute(BoundingBox(self))
    }
}

impl Modifier<Control> for Resource<BindingCanvas> {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::Canvas(self))
    }
}

impl Modifier<Control> for (ActionTrigger, String) {
    fn modify(self, control: &mut Control) {
        control.add_attribute(Action(self.0, self.1))
    }
}

impl<'a> Modifier<Control> for (ActionTrigger, &'a str) {
    fn modify(self, control: &mut Control) {
        control.add_attribute(Action(self.0, String::from(self.1)))
    }
}

impl Modifier<Control> for Vec<ControlAttribute> {
    fn modify(self, control: &mut Control) {
        for attr in self.into_iter() {
            control.add_attribute(attr);
        }
    }
}

impl Modifier<Control> for Vec<Control> {
    fn modify(self, control: &mut Control) {
        control.add_attribute(SubComponents(self))
    }
}

use super::bounds::*;
use super::control::*;
use super::actions::*;
use super::font_attr::*;
use super::hint_attr::*;
use super::state_attr::*;
use super::popup_attr::*;
use super::scroll_attr::*;
use super::appearance_attr::*;

use super::super::image;
use super::super::property::*;
use super::super::binding_canvas::*;
use super::super::resource_manager::*;

use ::modifier::*;

///
/// Attribute attached to a control
///
#[derive(Clone, PartialEq, Debug)]
pub enum ControlAttribute {
    // TODO: layout attribute for bounding box, zindex and padding
    /// The bounding box for this control
    BoundingBox(Bounds),

    // The z-index of this item. Items with higher z-indexes are displayed over the top of those with lower z-indexes
    ZIndex(u32),

    // The padding to surround the subcomponents of this control with. Values are 'left top' and 'right bottom'.
    Padding((u32, u32), (u32, u32)),

    /// The text for this control
    Text(Property),

    /// Specifies the font properties of this control
    FontAttr(Font),

    /// Specifies the state of this control
    StateAttr(State),

    /// Specifies the popup state of this control (meaningless if this is not a popup control)
    PopupAttr(Popup),

    /// Specifies the appearance of this control
    AppearanceAttr(Appearance),

    /// Specifies how the contents of this control will scroll
    ScrollAttr(Scroll),

    /// Specifies a hint on how this control should be treated
    HintAttr(Hint),

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
    Canvas(Resource<BindingCanvas>)
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
    /// The Z-Index represented by this attribute
    ///
    pub fn z_index(&self) -> Option<u32> {
        match self {
            &ZIndex(zindex) => Some(zindex),
            _               => None
        }
    }

    ///
    /// The padding represented by this attribute
    ///
    pub fn padding(&self) -> Option<((u32, u32), (u32, u32))> {
        match self {
            &Padding(left_top, right_bottom)    => Some((left_top, right_bottom)),
            _                                   => None
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
    pub fn font<'a>(&'a self) -> Option<&'a Font> {
        match self {
            &FontAttr(ref attr) => Some(attr),
            _                   => None
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
    /// The control state represented by this attribute
    ///
    pub fn state<'a>(&'a self) -> Option<&'a State> {
        match self {
            &StateAttr(ref state)   => Some(state),
            _                       => None
        }
    }

    ///
    /// The popup attribute represented by this attribute
    ///
    pub fn popup<'a>(&'a self) -> Option<&'a Popup> {
        match self {
            &PopupAttr(ref popup)   => Some(popup),
            _                       => None
        }
    }

    ///
    /// The canvas resource represented by this attribute, if there is one
    ///
    pub fn canvas<'a>(&'a self) -> Option<&'a Resource<BindingCanvas>> {
        match self {
            &Canvas(ref canvas) => Some(canvas),
            _                   => None
        }
    }

    ///
    /// The image resource represented by this attribute, if there is one
    ///
    pub fn image<'a>(&'a self) -> Option<&'a Resource<image::Image>> {
        match self {
            &AppearanceAttr(Appearance::Image(ref img)) => Some(img),
            _                                           => None
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
    /// The appearance assigned by this attribute, if there is one
    ///
    pub fn scroll<'a>(&'a self) -> Option<&'a Scroll> {
        match self {
            &ScrollAttr(ref scroll) => Some(scroll),
            _                       => None
        }
    }

    ///
    /// If this is a hint attribute, returns the hint, otherwise returns nothing
    ///
    pub fn hint<'a>(&'a self) -> Option<&'a Hint> {
        match self {
            &HintAttr(ref hint) => Some(hint),
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
            &ZIndex(zindex)                     => Some(zindex) != compare_to.z_index(),
            &Padding(lt, rb)                    => Some((lt, rb)) != compare_to.padding(),
            &Text(ref text)                     => Some(text) != compare_to.text(),
            &FontAttr(ref font)                 => Some(font) != compare_to.font(),
            &Id(ref id)                         => Some(id) != compare_to.id(),
            &Controller(ref controller)         => Some(controller.as_ref()) != compare_to.controller(),
            &Action(ref trigger, ref action)    => Some((trigger, action)) != compare_to.action(),
            &StateAttr(ref state)               => Some(state) != compare_to.state(),
            &PopupAttr(ref popup)               => Some(popup) != compare_to.popup(),
            &Canvas(ref canvas_resource)        => Some(canvas_resource) != compare_to.canvas(),
            &AppearanceAttr(ref appearance)     => Some(appearance) != compare_to.appearance(),
            &ScrollAttr(ref scroll)             => Some(scroll) != compare_to.scroll(),
            &HintAttr(ref hint)                 => Some(hint) != compare_to.hint(),

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

use super::diff::*;
use super::controller::*;

///
/// Attribute attached to a control
///
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlAttribute<TController: Controller> {
    /// The bounding box for this control
    BoundingBox(Bounds),

    /// The text for this control
    Text(String),

    /// The unique ID for this control
    Id(String),

    /// Subcomponents of this control
    SubComponents(Vec<Control<TController>>),

    /// Subcomponents are controlled by a particular controller
    ControlledBy(TController::ControllerSpecifier)
}

impl<TController: Controller> ControlAttribute<TController> {
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
    pub fn text<'a>(&'a self) -> Option<&'a String> {
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
    pub fn subcomponents<'a>(&'a self) -> Option<&'a Vec<Control<TController>>> {
        match self {
            &SubComponents(ref components)  => Some(components),
            _                               => None
        }
    }

    ///
    /// Returns true if this attribute is different from another one
    /// (non-recursively, so this won't check subcomoponents)
    ///
    pub fn is_different_flat(&self, compare_to: &ControlAttribute<TController>) -> bool {
        match self {
            &BoundingBox(ref bounds)        => Some(bounds) == compare_to.bounding_box(),
            &Text(ref text)                 => Some(text) == compare_to.text(),
            &Id(ref id)                     => Some(id) == compare_to.id(),

            // For the subcomponents we only care about the number as we don't want to recurse
            &SubComponents(ref components)  => Some(components.len()) == compare_to.subcomponents().map(|components| components.len())
        }
    }
}

use ControlAttribute::*;

///
/// Trait implemented by things that can be converted into control attributes
///
pub trait ToControlAttributes<TController: Controller> {
    fn attributes(&self) -> Vec<ControlAttribute<TController>>;
}

impl<TController: Controller> ToControlAttributes<TController> for ControlAttribute<TController> {
    fn attributes(&self) -> Vec<ControlAttribute<TController>> {
        vec![self.clone()]
    }
}

impl<'a, TController: Controller> ToControlAttributes<TController> for &'a str {
    fn attributes(&self) -> Vec<ControlAttribute<TController>> {
        vec![Text(String::from(*self))]
    }
}

impl<TController: Controller> ToControlAttributes<TController> for Bounds {
    fn attributes(&self) -> Vec<ControlAttribute<TController>> {
        vec![BoundingBox(self.clone())]
    }
}

impl<TController: Controller> ToControlAttributes<TController> for Vec<ControlAttribute<TController>> {
    fn attributes(&self) -> Vec<ControlAttribute<TController>> {
        self.clone()
    }
}

impl<TController: Controller, A: ToControlAttributes<TController>, B: ToControlAttributes<TController>> ToControlAttributes<TController> for (A, B) {
    fn attributes(&self) -> Vec<ControlAttribute<TController>> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());

        res
    }
}

impl<TController: Controller, A: ToControlAttributes<TController>, B: ToControlAttributes<TController>, C: ToControlAttributes<TController>> ToControlAttributes<TController> for (A, B, C) {
    fn attributes(&self) -> Vec<ControlAttribute<TController>> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());
        res.append(&mut self.2.attributes());

        res
    }
}

impl<TController: Controller, A: ToControlAttributes<TController>, B: ToControlAttributes<TController>, C: ToControlAttributes<TController>, D: ToControlAttributes<TController>> ToControlAttributes<TController> for (A, B, C, D) {
    fn attributes(&self) -> Vec<ControlAttribute<TController>> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());
        res.append(&mut self.2.attributes());
        res.append(&mut self.3.attributes());

        res
    }
}

impl<TController: Controller> ToControlAttributes<TController> for Vec<Control<TController>> {
    fn attributes(&self) -> Vec<ControlAttribute<TController>> {
        vec![SubComponents(self.iter().cloned().collect())]
    }
}

///
/// Possible types of control
///
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlType {
    /// A control that does nothing
    Empty,

    /// Control that contains other controls
    Container,

    /// Clickable button
    Button,

    /// Label used to display some text
    Label
}

use ControlType::*;

///
/// Represents a control
///
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Control<TController: Controller> {
    /// Attributes for this control
    attributes: Vec<ControlAttribute<TController>>,

    /// Type of this control
    control_type: ControlType
}

impl<TController: Controller> Control<TController> {
    /// Creates a new control of a particular type
    pub fn new(control_type: ControlType) -> Control<TController> {
        Control { attributes: vec![], control_type: control_type }
    }

    /// Creates a new container control
    pub fn container() -> Control<TController> {
        Self::new(Container)
    }

    /// Creates a new button control
    pub fn button() -> Control<TController> {
        Self::new(Button)
    }

    /// Creates a new label control
    pub fn label() -> Control<TController> {
        Self::new(Label)
    }

    /// Create a new empty control
    pub fn empty() -> Control<TController> {
        Self::new(Empty)
    }

    /// Creates a control with some attributes added to it
    pub fn with<T: ToControlAttributes<TController>>(&self, attributes: T) -> Control<TController> {
        let mut new_attributes = self.attributes.clone();
        new_attributes.append(&mut attributes.attributes());

        Control { attributes: new_attributes, control_type: self.control_type }
    }

    /// Returns an iterator over the attributes for this control
    pub fn attributes<'a>(&'a self) -> Box<Iterator<Item=&'a ControlAttribute<TController>>+'a> {
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
    pub fn has_attribute_flat(&self, attr: &ControlAttribute<TController>) -> bool {
        self.attributes.iter()
            .any(|test_attr| !test_attr.is_different_flat(attr))
    }
}

impl<TController: Controller> DiffableTree for Control<TController> {
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

///
/// Represents a position coordinate
///
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Position {
    /// Point located at a specific value
    At(f32),

    /// Point at an offset from its counterpart (eg, width or height)
    Offset(f32),

    /// As a final point, stretches with the specified ratio to other stretch controls
    Stretch(f32),

    /// Point located at the start of the container (ie, left or top depending on if this is an x or y position)
    Start,

    /// Control located at the end of its container (ie, right or bottom depending on if this is an x or y position)
    End,

    /// Same as the last point in this axis (which is 0 initially)
    After
}

///
/// Represents the bounds of a particular control
///
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub x1: Position,
    pub y1: Position,
    pub x2: Position,
    pub y2: Position
}

impl Bounds {
    ///
    /// Creates a bounding box that fills a container
    ///
    pub fn fill_all() -> Bounds {
        use Position::*;
        Bounds { x1: Start, y1: Start, x2: End, y2: End }
    }

    ///
    /// Bounding box that fills the container vertically and follows the previous control horizontally
    ///
    pub fn next_horiz(width: f32) -> Bounds {
        use Position::*;
        Bounds { x1: After, y1: Start, x2: Offset(width), y2: End }
    }

    ///
    /// Bounding box that fills the container horizontally and follows the previous control horizontally
    ///
    pub fn next_vert(height: f32) -> Bounds {
        use Position::*;
        Bounds { x1: Start, y1: After, x2: End, y2: Offset(height) }
    }

    ///
    /// Bounding box that fills the remaining horizontal space
    ///
    pub fn fill_horiz() -> Bounds {
        use Position::*;
        Bounds { x1: After, y1: Start, x2: End, y2: End }
    }

    ///
    /// Bounding box that fills the remaining vertical space
    ///
    pub fn fill_vert() -> Bounds {
        use Position::*;
        Bounds { x1: Start, y1: After, x2: End, y2: End }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_button() {
        let button = Control::button();

        assert!(button.control_type() == ControlType::Button);
    }

    #[test]
    fn can_create_label_with_text() {
        let label = Control::label().with("Hello");

        assert!(label.control_type() == ControlType::Label);
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::Text(String::from("Hello"))));
    }

    #[test]
    fn can_create_label_with_many_attributes() {
        let label = Control::label().with(("Hello", Bounds::fill_all()));

        assert!(label.control_type() == ControlType::Label);
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::Text(String::from("Hello"))));
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::BoundingBox(Bounds::fill_all())));
    }

    #[test]
    fn can_create_container_with_components() {
        let container = Control::container()
            .with(vec![Control::label().with("Hello")]);

        assert!(container.control_type() == ControlType::Container);
        assert!(container.attributes().any(|attr| attr == &ControlAttribute::SubComponents(vec![Control::label().with("Hello")])));
    }
}

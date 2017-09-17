///
/// Attributes can be attached to animations and frames in order to
/// provide additional information about them. Attributes are generally
/// optional, and provide an easily extensible way to add extra properties
/// to something.
///
pub enum Attribute {
    /// Attribute representing the name of this item
    Name(String)
}

///
/// Anything with attributes can implement the HasAttributes trait
///
pub trait HasAttributes {
    ///
    /// Retrieves the attributes attached to this item
    ///
    fn attributes(&self) -> Box<Iterator<Item = Attribute>>;
}
///
/// How a set of elements in a group are combined
///
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum GroupType {
    /// Elements are just rendered one after the other
    Normal,

    /// Elements are added together (the path properties of the first element are used for all elements)
    Added,

    /*
    /// Elements after the first element are subtracted from the first element
    Subtracted,

    /// The first element is intersected with future elements
    Masked,

    /// The first element is subtracted from future elements
    InvertedMask
    */
}

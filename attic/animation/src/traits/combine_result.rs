use super::vector::*;

///
/// Result of attempting to combine two vector elements
///
pub enum CombineResult {
    /// A successful combination
    NewElement(Vector),

    /// The two elements do not overlap (further elements might overlap, so continue to combine elements)
    NoOverlap,

    /// The two elements overlap but cannot be combined
    CannotCombineAndOverlaps,

    /// The two elements cannot be combined and no element underneath the second element can be combined with the first
    UnableToCombineFurther
}

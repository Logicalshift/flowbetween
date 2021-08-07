///
/// A region is defined by two values: the index of the animation region, and the index of the sub-region within
/// that region.
///
/// (Ie, we can divide a layer into multiple regions and each of those regions can have a number of sub-regions)
///
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct RegionId(pub usize, pub usize);

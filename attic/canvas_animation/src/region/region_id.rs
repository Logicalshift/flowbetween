///
/// Identifier of an animation region
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionId(pub usize);

impl RegionId {
    ///
    /// Returns the index of this region
    ///
    pub fn index(&self) -> usize { self.0 }
}

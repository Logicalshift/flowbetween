///
/// The index of a path within an animation layer layer
///
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PathIndex(pub usize);

impl PathIndex {
    #[inline]
    pub fn idx(&self) -> usize { self.0 }
}
use super::super::traits::*;

///
/// Represents animation attributes that are stored in memory
/// 
pub struct InMemoryAttributeSet {
    /// The attributes in this set
    attributes: Vec<Box<AnimationAttribute>>
}

impl HasAttributes for InMemoryAttributeSet {
    fn attributes<'a>(&'a self) -> Box<'a+Iterator<Item = &'a AnimationAttribute>> {
        Box::new(self.attributes.iter().map(|attr| &**attr))
    }
}

use super::super::vector::*;

///
/// Represents a layer that contains vector elements
/// 
pub trait VectorLayer : Send+Sync {
    ///
    /// Retrieves the elements from this layer
    /// 
    fn elements<'a>(&'a self) -> Box<'a+Iterator<Item=&VectorElement>>;

    ///
    /// Adds a new vector element to this layer
    /// 
    fn add_element(&mut self, new_element: Box<VectorElement>);
}

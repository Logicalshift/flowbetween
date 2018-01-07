use super::*;

impl VectorLayer for VectorLayerCore {
    ///
    /// Retrieves all of the elements from this layer
    /// 
    fn elements<'a>(&'a self) -> Box<'a+Iterator<Item=&VectorElement>> {
        Box::new(self.elements.iter()
            .flat_map(|(_k, v)| v.iter())
            .map(|element| &**element))
    }

    ///
    /// Adds a new vector element to this layer
    /// 
    fn add_element(&mut self, new_element: Box<VectorElement>) {
        self.elements
            .entry(new_element.appearance_time()).or_insert_with(|| vec![])
            .push(new_element)
    }
}

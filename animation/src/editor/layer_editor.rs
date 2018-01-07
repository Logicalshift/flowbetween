use super::super::traits::*;

///
/// Performs edits on a layer
/// 
pub struct LayerEditor {
}

impl LayerEditor {
    ///
    /// Creates a new animation editor
    /// 
    pub fn new() -> LayerEditor {
        LayerEditor { }
    }

    ///
    /// Performs some edits on the specified layer
    /// 
    pub fn perform<Edits: IntoIterator<Item=LayerEdit>>(&self, _target: &mut Layer, _edits: Edits) {
        unimplemented!()
    }
}

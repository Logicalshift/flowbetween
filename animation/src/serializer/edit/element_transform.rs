use super::super::source::*;
use super::super::target::*;

use crate::traits::*;

impl ElementTransform {
    ///
    /// Generates a serialized version of this transformation on a data target
    ///
    pub fn serialize<Tgt: AnimationDataTarget>(&self, _data: &mut Tgt) {
        match self {
            _ => { unimplemented!() }            
        }
    }

    ///
    /// Reads an element transformation from a data source
    ///
    pub fn deserialize<Src: AnimationDataSource>(_data: &mut Src) -> Option<ElementTransform> {
        None
    }
}

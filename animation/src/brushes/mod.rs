mod simple;
mod ink;

pub use self::simple::*;
pub use self::ink::*;

use super::traits::*;
use std::sync::*;

///
/// Creates a brush from a brush definition
/// 
pub fn create_brush_from_definition(definition: &BrushDefinition) -> Arc<Brush> {
    use BrushDefinition::*;

    match definition {
        &Ink(ref ink_definition) => Arc::new(InkBrush::new(ink_definition, false))
    }
}

///
/// Creates an eraser from a brush definition
/// 
pub fn create_eraser_from_definition(definition: &BrushDefinition) -> Arc<Brush> {
    use BrushDefinition::*;

    match definition {
        &Ink(ref ink_definition) => Arc::new(InkBrush::new(ink_definition, true))
    }
}

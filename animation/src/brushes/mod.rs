mod simple;
mod ink;
mod preview;

pub use self::simple::*;
pub use self::ink::*;
pub use self::preview::*;

use super::traits::*;
use std::sync::*;

///
/// Creates a brush from a brush definition
/// 
pub fn create_brush_from_definition(definition: &BrushDefinition, drawing_style: BrushDrawingStyle) -> Arc<Brush> {
    use BrushDefinition::*;

    match definition {
        &Simple                     => Arc::new(SimpleBrush::new()),
        &Ink(ref ink_definition)    => Arc::new(InkBrush::new(ink_definition, drawing_style))
    }
}

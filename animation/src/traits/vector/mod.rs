use canvas::*;
use std::time::Duration;

mod properties;
mod path;
mod element;
mod brush_element;

pub use self::properties::*;
pub use self::path::*;
pub use self::element::*;
pub use self::brush_element::*;

///
/// Possible types of vector element
/// 
#[derive(Clone)]
pub enum Vector {
    /// Empty vector
    Empty,

    /// Brush stroke vector
    Brush(BrushElement)
}

impl VectorElement for Vector {
    fn appearance_time(&self) -> Duration {
        match self {
            &Vector::Empty              => Duration::from_millis(0),
            &Vector::Brush(ref elem)    => elem.appearance_time()
        }
    }

    fn path(&self) -> Path {
        match self {
            &Vector::Empty              => Path::new(),
            &Vector::Brush(ref elem)    => elem.path()
        }
    }

    fn render(&self, gc: &mut GraphicsPrimitives) {
        match self {
            &Vector::Empty              => (),
            &Vector::Brush(ref elem)    => elem.render(gc)
        }
    }
}

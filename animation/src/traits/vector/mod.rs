use ui::canvas::*;
use std::time::Duration;

mod path;
mod element;
mod brush_element;

pub use self::path::*;
pub use self::element::*;
pub use self::brush_element::*;

///
/// Possible types of vector element
/// 
#[derive(Clone)]
pub enum Vector {
    Brush(BrushElement)
}

impl VectorElement for Vector {
    fn appearance_time(&self) -> Duration {
        match self {
            &Vector::Brush(ref elem) => elem.appearance_time()
        }
    }

    fn path(&self) -> Path {
        match self {
            &Vector::Brush(ref elem) => elem.path()
        }
    }

    fn render(&self, gc: &mut GraphicsContext) {
        match self {
            &Vector::Brush(ref elem) => elem.render(gc)
        }
    }
}

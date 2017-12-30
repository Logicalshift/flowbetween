use canvas::*;
use binding::*;

///
/// View model for the brush properties
/// 
#[derive(Clone)]
pub struct BrushViewModel {
    pub size: Binding<f32>,

    pub opacity: Binding<f32>,

    pub color: Binding<Color>
}

impl BrushViewModel {
    pub fn new() -> BrushViewModel {
        BrushViewModel {
            size:       bind(10.0),
            opacity:    bind(1.0),
            color:      bind(Color::Rgba(0.0, 0.0, 0.0, 1.0))
        }
    }
}

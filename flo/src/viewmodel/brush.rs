use canvas::*;
use binding::*;
use animation::*;

use std::sync::*;

///
/// View model for the brush properties
/// 
#[derive(Clone)]
pub struct BrushViewModel {
    /// The size of the brush (pixels)
    pub size: Binding<f32>,

    /// The opacity of the brush (0-1)
    pub opacity: Binding<f32>,

    /// The colour of the brush (in general alpha should be left at 1.0 here)
    pub color: Binding<Color>,

    /// The brush properties for the current brush view model
    pub brush_properties: Arc<Bound<BrushProperties>>
}

impl BrushViewModel {
    ///
    /// Creates a new brush view model
    /// 
    pub fn new() -> BrushViewModel {
        let size                = bind(10.0);
        let opacity             = bind(1.0);
        let color               = bind(Color::Rgba(0.0, 0.0, 0.0, 1.0));

        let brush_properties    = Self::brush_properties(size.clone(), opacity.clone(), color.clone());

        BrushViewModel {
            size:               size,
            opacity:            opacity,
            color:              color,
            brush_properties:   brush_properties
        }
    }

    fn brush_properties(size: Binding<f32>, opacity: Binding<f32>, color: Binding<Color>) -> Arc<Bound<BrushProperties>> {
        let brush_properties = computed(move || {
            BrushProperties {
                size:       size.get(),
                opacity:    opacity.get(),
                color:      color.get()
            }
        });

        Arc::new(brush_properties)
    }
}

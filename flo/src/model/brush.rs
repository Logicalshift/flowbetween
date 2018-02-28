use canvas::*;
use binding::*;
use animation::*;

///
/// View model for the brush properties
/// 
#[derive(Clone)]
pub struct BrushModel {
    /// The size of the brush (pixels)
    pub size: Binding<f32>,

    /// The opacity of the brush (0-1)
    pub opacity: Binding<f32>,

    /// The colour of the brush (in general alpha should be left at 1.0 here)
    pub color: Binding<Color>,

    /// The brush properties for the current brush view model
    pub brush_properties: BindRef<BrushProperties>
}

impl BrushModel {
    ///
    /// Creates a new brush view model
    /// 
    pub fn new() -> BrushModel {
        let size                = bind(5.0);
        let opacity             = bind(1.0);
        let color               = bind(Color::Rgba(0.0, 0.0, 0.0, 1.0));

        let brush_properties    = Self::brush_properties(size.clone(), opacity.clone(), color.clone());

        BrushModel {
            size:               size,
            opacity:            opacity,
            color:              color,
            brush_properties:   brush_properties
        }
    }

    fn brush_properties(size: Binding<f32>, opacity: Binding<f32>, color: Binding<Color>) -> BindRef<BrushProperties> {
        let brush_properties = computed(move || {
            BrushProperties {
                size:       size.get(),
                opacity:    opacity.get(),
                color:      color.get()
            }
        });

        BindRef::from(brush_properties)
    }
}

use canvas::*;

mod properties;
mod path;
mod element;
mod brush_element;
mod brush_properties_element;
mod brush_definition_element;

pub use self::properties::*;
pub use self::path::*;
pub use self::element::*;
pub use self::brush_element::*;
pub use self::brush_properties_element::*;
pub use self::brush_definition_element::*;

///
/// Possible types of vector element
/// 
#[derive(Clone)]
pub enum Vector {
    /// Sets the brush properties for future brush strokes
    BrushDefinition(BrushDefinitionElement),

    /// Brush properties for future brush strokes
    BrushProperties(BrushPropertiesElement),

    /// Brush stroke vector
    BrushStroke(BrushElement)
}

impl Vector {
    ///
    /// Creates a new vector from an element
    /// 
    #[inline]
    pub fn new<IntoVec: Into<Vector>>(from: IntoVec) -> Vector {
        from.into()
    }
}

impl VectorElement for Vector {
    fn render(&self, gc: &mut GraphicsPrimitives, properties: &VectorProperties) {
        use Vector::*;

        match self {
            &BrushDefinition(ref defn)  => defn.render(gc, properties),
            &BrushProperties(ref props) => props.render(gc, properties),
            &BrushStroke(ref elem)      => elem.render(gc, properties)
        }
    }

    fn update_properties(&self, properties: &mut VectorProperties) { 
        use Vector::*;

        match self {
            &BrushDefinition(ref defn)  => defn.update_properties(properties),
            &BrushProperties(ref props) => props.update_properties(properties),
            &BrushStroke(ref elem)      => elem.update_properties(properties)
        }
    }
}

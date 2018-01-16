///
/// Type of edit log item
/// 
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum EditLogType {
    SetSize,
    AddNewLayer,
    RemoveLayer,

    LayerAddKeyFrame,
    LayerRemoveKeyFrame,

    LayerPaintSelectBrush,
    LayerPaintBrushProperties,
    LayerPaintBrushStroke
}

///
/// Types of drawing style
/// 
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum DrawingStyleType {
    Draw,
    Erase
}

///
/// Types of brush definition
/// 
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum BrushDefinitionType {
    Simple,
    Ink
}

///
/// Types of colour definition
/// 
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum ColorType {
    Rgb,
    Hsluv
}

///
/// Types of player
/// 
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum LayerType {
    Vector
}

///
/// Types of vector element
/// 
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum VectorElementType {
    BrushDefinition,
    BrushProperties,
    BrushStroke
}

///
/// All of the DB enums in one place
/// 
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum DbEnum {
    EditLog(EditLogType),
    DrawingStyle(DrawingStyleType),
    BrushDefinition(BrushDefinitionType),
    Color(ColorType),
    Layer(LayerType),
    VectorElement(VectorElementType)
}
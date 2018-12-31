use super::motion_path_type::*;

use flo_animation::*;

/// Provides the enum type and name for a database enum value
pub struct DbEnumName(pub &'static str, pub &'static str);

///
/// Type of edit log item
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EditLogType {
    SetSize,
    AddNewLayer,
    RemoveLayer,

    LayerAddKeyFrame,
    LayerRemoveKeyFrame,
    LayerSetName,

    LayerPaintSelectBrush,
    LayerPaintBrushProperties,
    LayerPaintBrushStroke,

    LayerPathCreatePath,
    LayerPathSelectBrush,
    LayerPathBrushProperties,

    MotionCreate,
    MotionDelete,
    MotionSetType,
    MotionSetOrigin,
    MotionSetPath,
    MotionAttach,
    MotionDetach,

    ElementSetControlPoints
}

///
/// Types of drawing style
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DrawingStyleType {
    Draw,
    Erase
}

///
/// Types of brush definition
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum BrushDefinitionType {
    Simple,
    Ink
}

///
/// Types of colour definition
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ColorType {
    Rgb,
    Hsluv
}

///
/// Types of player
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum LayerType {
    Vector
}

///
/// Types of vector element
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum VectorElementType {
    BrushDefinition,
    BrushProperties,
    BrushStroke,
    Path
}

///
/// Types of path point
///
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PathPointType {
    MoveTo,
    LineTo,
    ControlPoint,
    BezierTo,
    Close
}

///
/// All of the DB enums in one place
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DbEnum {
    EditLog(EditLogType),
    DrawingStyle(DrawingStyleType),
    BrushDefinition(BrushDefinitionType),
    Color(ColorType),
    Layer(LayerType),
    MotionType(MotionType),
    MotionPathType(MotionPathType),
    VectorElement(VectorElementType),
    PathPoint(PathPointType)
}

impl DbEnum {
    /// Returns the EditLog value for this enum (if there is one)
    pub fn edit_log(self) -> Option<EditLogType> {
        match self {
            DbEnum::EditLog(res)    => Some(res),
            _                       => None
        }
    }

    /// Returns the DrawingStyle value for this enum (if there is one)
    pub fn drawing_style(self) -> Option<DrawingStyleType> {
        match self {
            DbEnum::DrawingStyle(res)   => Some(res),
            _                           => None
        }
    }

    /// Returns the BrushDefinition value for this enum (if there is one)
    pub fn brush_definition(self) -> Option<BrushDefinitionType> {
        match self {
            DbEnum::BrushDefinition(res)    => Some(res),
            _                               => None
        }
    }

    /// Returns the Color value for this enum (if there is one)
    pub fn color(self) -> Option<ColorType> {
        match self {
            DbEnum::Color(res)  => Some(res),
            _                   => None
        }
    }

    /// Returns the Layer value for this enum (if there is one)
    pub fn layer(self) -> Option<LayerType> {
        match self {
            DbEnum::Layer(res)  => Some(res),
            _                   => None
        }
    }

    /// Returns the VectorElement value for this enum (if there is one)
    pub fn vector_element(self) -> Option<VectorElementType> {
        match self {
            DbEnum::VectorElement(res)  => Some(res),
            _                           => None
        }
    }

    /// Returns the MotionType value for this enum (if there is one)
    pub fn motion_type(self) -> Option<MotionType> {
        match self {
            DbEnum::MotionType(res) => Some(res),
            _                       => None
        }
    }
}

///
/// The types of enumeration that are in the database
/// 
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DbEnumType {
    EditLog,
    DrawingStyle,
    BrushDefinition,
    Color,
    Layer,
    VectorElement,
    MotionType,
    PathPoint
}

impl From<DbEnumType> for Vec<DbEnum> {
    fn from(t: DbEnumType) -> Vec<DbEnum> {
        use self::DbEnumType::*;

        match t {
            EditLog => {
                use self::EditLogType::*;
                vec![
                    DbEnum::EditLog(SetSize),
                    DbEnum::EditLog(AddNewLayer),
                    DbEnum::EditLog(RemoveLayer),

                    DbEnum::EditLog(LayerAddKeyFrame),
                    DbEnum::EditLog(LayerRemoveKeyFrame),
                    DbEnum::EditLog(LayerSetName),

                    DbEnum::EditLog(LayerPaintSelectBrush),
                    DbEnum::EditLog(LayerPaintBrushProperties),
                    DbEnum::EditLog(LayerPaintBrushStroke),

                    DbEnum::EditLog(LayerPathCreatePath),
                    DbEnum::EditLog(LayerPathSelectBrush),
                    DbEnum::EditLog(LayerPathBrushProperties),

                    DbEnum::EditLog(MotionCreate),
                    DbEnum::EditLog(MotionDelete),
                    DbEnum::EditLog(MotionSetType),
                    DbEnum::EditLog(MotionSetOrigin),
                    DbEnum::EditLog(MotionSetPath),
                    DbEnum::EditLog(MotionAttach),
                    DbEnum::EditLog(MotionDetach),

                    DbEnum::EditLog(ElementSetControlPoints)
                ]
            },

            DrawingStyle => {
                use self::DrawingStyleType::*;
                vec![
                    DbEnum::DrawingStyle(Draw),
                    DbEnum::DrawingStyle(Erase)
                ]
            },

            BrushDefinition => {
                use self::BrushDefinitionType::*;
                vec![
                    DbEnum::BrushDefinition(Simple),
                    DbEnum::BrushDefinition(Ink)
                ]
            },

            Color => {
                use self::ColorType::*;
                vec![
                    DbEnum::Color(Rgb),
                    DbEnum::Color(Hsluv)
                ]
            },

            Layer => {
                use self::LayerType::*;
                vec![
                    DbEnum::Layer(Vector)
                ]
            },

            VectorElement => {
                use self::VectorElementType::*;
                vec![
                    DbEnum::VectorElement(BrushDefinition),
                    DbEnum::VectorElement(BrushProperties),
                    DbEnum::VectorElement(BrushStroke),
                    DbEnum::VectorElement(Path)
                ]
            },

            MotionType => {
                use self::MotionType::*;

                vec![
                    DbEnum::MotionType(None),
                    DbEnum::MotionType(Translate)
                ]
            },

            PathPoint => {
                use self::PathPointType::*;

                vec![
                    DbEnum::PathPoint(MoveTo),
                    DbEnum::PathPoint(LineTo),
                    DbEnum::PathPoint(ControlPoint),
                    DbEnum::PathPoint(BezierTo),
                    DbEnum::PathPoint(Close)
                ]
            }
        }
    }
}

impl<'a> From<&'a AnimationEdit> for EditLogType {
    fn from(t: &AnimationEdit) -> EditLogType {
        use self::AnimationEdit::*;
        use self::ElementEdit::*;
        use self::MotionEdit::*;
        use self::LayerEdit::*;
        use self::PaintEdit::*;
        use self::PathEdit::CreatePath;

        match t {
            SetSize(_, _)                                       => EditLogType::SetSize,
            AddNewLayer(_)                                      => EditLogType::AddNewLayer,
            RemoveLayer(_)                                      => EditLogType::RemoveLayer,
            
            Layer(_, AddKeyFrame(_))                            => EditLogType::LayerAddKeyFrame,
            Layer(_, RemoveKeyFrame(_))                         => EditLogType::LayerRemoveKeyFrame,
            Layer(_, SetName(_))                                => EditLogType::LayerSetName,
            Layer(_, Paint(_, SelectBrush(_, _, _)))            => EditLogType::LayerPaintSelectBrush,
            Layer(_, Paint(_, BrushProperties(_, _)))           => EditLogType::LayerPaintBrushProperties,
            Layer(_, Paint(_, BrushStroke(_,_)))                => EditLogType::LayerPaintBrushStroke,

            Layer(_, Path(_, CreatePath(_, _)))                 => EditLogType::LayerPathCreatePath,
            Layer(_, Path(_, PathEdit::SelectBrush(_, _, _)))   => EditLogType::LayerPathSelectBrush,
            Layer(_, Path(_, PathEdit::BrushProperties(_, _)))  => EditLogType::LayerPathBrushProperties,

            Motion(_, Create)                                   => EditLogType::MotionCreate,
            Motion(_, Delete)                                   => EditLogType::MotionDelete,
            Motion(_, SetType(_))                               => EditLogType::MotionSetType,
            Motion(_, SetOrigin(_, _))                          => EditLogType::MotionSetOrigin,
            Motion(_, SetPath(_))                               => EditLogType::MotionSetPath,
            Motion(_, Attach(_))                                => EditLogType::MotionAttach,
            Motion(_, Detach(_))                                => EditLogType::MotionDetach,

            Element(_, SetControlPoints(_))                     => EditLogType::ElementSetControlPoints
        }
    }
}

impl<'a> From<&'a BrushDrawingStyle> for DrawingStyleType {
    fn from(t: &BrushDrawingStyle) -> DrawingStyleType {
        use self::BrushDrawingStyle::*;

        match t {
            &Draw   => DrawingStyleType::Draw,
            &Erase  => DrawingStyleType::Erase
        }
    }
}

impl Into<BrushDrawingStyle> for DrawingStyleType {
    fn into(self) -> BrushDrawingStyle {
        match self {
            DrawingStyleType::Draw  => BrushDrawingStyle::Draw,
            DrawingStyleType::Erase => BrushDrawingStyle::Erase
        }
    }
}

impl<'a> From<&'a PaintEdit> for VectorElementType {
    fn from(t: &PaintEdit) -> VectorElementType {
        use self::PaintEdit::*;

        match t {
            SelectBrush(_, _, _)    => VectorElementType::BrushDefinition,
            BrushProperties(_, _)   => VectorElementType::BrushProperties,
            BrushStroke(_, _)       => VectorElementType::BrushStroke
        }
    }
}

impl<'a> From<&'a BrushDefinition> for BrushDefinitionType {
    fn from(t: &BrushDefinition) -> BrushDefinitionType {
        use self::BrushDefinition::*;

        match t {
            &Simple     => BrushDefinitionType::Simple,
            &Ink(_)     => BrushDefinitionType::Ink
        }
    }
}


impl From<EditLogType> for DbEnumName {
    fn from(t: EditLogType) -> DbEnumName {
        use self::EditLogType::*;

        match t {
            SetSize                     => DbEnumName("Edit", "SetSize"),
            AddNewLayer                 => DbEnumName("Edit", "AddNewLayer"),
            RemoveLayer                 => DbEnumName("Edit", "RemoveLayer"),

            LayerAddKeyFrame            => DbEnumName("Edit", "Layer::AddKeyFrame"),
            LayerRemoveKeyFrame         => DbEnumName("Edit", "Layer::RemoveKeyFrame"),
            LayerSetName                => DbEnumName("Edit", "Layer::SetName"),

            LayerPaintSelectBrush       => DbEnumName("Edit", "Layer::Paint::SelectBrush"),
            LayerPaintBrushProperties   => DbEnumName("Edit", "Layer::Paint::BrushProperties"),
            LayerPaintBrushStroke       => DbEnumName("Edit", "Layer::Paint::BrushStroke"),

            LayerPathCreatePath         => DbEnumName("Edit", "Layer::Path::CreatePath"),
            LayerPathSelectBrush        => DbEnumName("Edit", "Layer::Path::SelectBrush"),
            LayerPathBrushProperties    => DbEnumName("Edit", "Layer::Path::BrushProperties"),

            MotionCreate                => DbEnumName("Edit", "Motion::Create"),
            MotionDelete                => DbEnumName("Edit", "Motion::Delete"),
            MotionSetType               => DbEnumName("Edit", "Motion::SetType"),
            MotionSetOrigin             => DbEnumName("Edit", "Motion::SetOrigin"),
            MotionSetPath               => DbEnumName("Edit", "Motion::SetPath"),
            MotionAttach                => DbEnumName("Edit", "Motion::Attach"),
            MotionDetach                => DbEnumName("Edit", "Motion::Detach"),

            ElementSetControlPoints     => DbEnumName("Edit", "Element::SetControlPoints")
        }
    }
}

impl From<DrawingStyleType> for DbEnumName {
    fn from(t: DrawingStyleType) -> DbEnumName {
        use self::DrawingStyleType::*;

        match t {
            Draw    => DbEnumName("DrawingStyle", "Draw"),
            Erase   => DbEnumName("DrawingStyle", "Erase")
        }
    }
}

impl From<BrushDefinitionType> for DbEnumName {
    fn from(t: BrushDefinitionType) -> DbEnumName {
        use self::BrushDefinitionType::*;

        match t {
            Simple  => DbEnumName("BrushType", "Simple"),
            Ink     => DbEnumName("BrushType", "Ink")
        }
    }
}

impl From<ColorType> for DbEnumName {
    fn from(t: ColorType) -> DbEnumName {
        use self::ColorType::*;

        match t {
            Rgb     => DbEnumName("ColorType", "RGB"),
            Hsluv   => DbEnumName("ColorType", "HSLUV")
        }
    }
}

impl From<LayerType> for DbEnumName {
    fn from(t: LayerType) -> DbEnumName {
        use self::LayerType::*;

        match t {
            Vector  => DbEnumName("LayerType", "Vector")
        }
    }
}

impl From<VectorElementType> for DbEnumName {
    fn from(t: VectorElementType) -> DbEnumName {
        use self::VectorElementType::*;

        match t {
            BrushDefinition     => DbEnumName("VectorElementType", "BrushDefinition"),
            BrushProperties     => DbEnumName("VectorElementType", "BrushProperties"),
            BrushStroke         => DbEnumName("VectorElementType", "BrushStroke"),
            Path                => DbEnumName("VectorElementType", "Path"),
        }
    }
}

impl From<MotionType> for DbEnumName {
    fn from(t: MotionType) -> DbEnumName {
        use self::MotionType::*;

        match t {
            None        => DbEnumName("MotionType", "None"),
            Reverse     => DbEnumName("MotionType", "Reverse"),
            Translate   => DbEnumName("MotionType", "Translate")
        }
    }
}

impl From<MotionPathType> for DbEnumName {
    fn from(t: MotionPathType) -> DbEnumName {
        use self::MotionPathType::*;

        match t {
            Position    => DbEnumName("MotionPathType", "Position"),
        }
    }
}

impl From<PathPointType> for DbEnumName {
    fn from(t: PathPointType) -> DbEnumName {
        use self::PathPointType::*;

        match t {
            MoveTo          => DbEnumName("PathPointType", "MoveTo"),
            LineTo          => DbEnumName("PathPointType", "LineTo"),
            ControlPoint    => DbEnumName("PathPointType", "ControlPoint"),
            BezierTo        => DbEnumName("PathPointType", "BezierTo"),
            Close           => DbEnumName("PathPointType", "Close")
        }
    }
}

impl From<DbEnum> for DbEnumName {
    fn from(t: DbEnum) -> DbEnumName {
        use self::DbEnum::*;

        match t {
            EditLog(elt)            => DbEnumName::from(elt),
            DrawingStyle(dst)       => DbEnumName::from(dst),
            BrushDefinition(bdt)    => DbEnumName::from(bdt),
            Color(ct)               => DbEnumName::from(ct),
            Layer(lt)               => DbEnumName::from(lt),
            VectorElement(vet)      => DbEnumName::from(vet),
            MotionType(mot)         => DbEnumName::from(mot),
            MotionPathType(mpt)     => DbEnumName::from(mpt),
            PathPoint(ppt)          => DbEnumName::from(ppt)
        }
    }
}
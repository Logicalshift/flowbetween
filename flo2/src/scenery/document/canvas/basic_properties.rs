use super::property::*;

use flo_draw::canvas::*;

use ::serde::*;

static PROP_FILL_COLOR_TYPE: LazyCanvasPropertyId   = LazyCanvasPropertyId::new("flowbetween::fill_color_type");
static PROP_FILL_COLOR: LazyCanvasPropertyId        = LazyCanvasPropertyId::new("flowbetween::fill_color");
static PROP_STROKE_COLOR_TYPE: LazyCanvasPropertyId = LazyCanvasPropertyId::new("flowbetween::stroke_color_type");
static PROP_STROKE_COLOR: LazyCanvasPropertyId      = LazyCanvasPropertyId::new("flowbetween::stroke_color");
static PROP_STROKE_LINECAP: LazyCanvasPropertyId    = LazyCanvasPropertyId::new("flowbetween::stroke_linecap");
static PROP_STROKE_LINEJOIN: LazyCanvasPropertyId   = LazyCanvasPropertyId::new("flowbetween::stroke_linejoin");
static PROP_STROKE_WIDTH: LazyCanvasPropertyId      = LazyCanvasPropertyId::new("flowbetween::stroke_width");

///
/// Property applied to a shape that should have a flat fill
///
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FlatFill(pub Color);

///
/// Width of a stroke
///
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokeWidth(pub f64);

///
/// Property applied to a shape that should be surrounded by a line stroke
///
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stroke(pub StrokeWidth, pub LineCap, pub LineJoin, pub Color);

///
/// Returns the type of a color property (a color has a type and a value property)
///
pub fn color_type_property(color: &Color) -> CanvasProperty {
    match color {
        Color::Rgba(_, _, _, _)     => CanvasProperty::Int(0),
        Color::Hsluv(_, _, _, _)    => CanvasProperty::Int(1),
    }
}

///
/// Returns the value property for a color
///
pub fn color_value_property(color: &Color) -> CanvasProperty {
    match color {
        Color::Rgba(r, g, b, a)     => CanvasProperty::FloatList(vec![*r as _, *g as _, *b as _, *a as _]),
        Color::Hsluv(h, s, l, a)    => CanvasProperty::FloatList(vec![*h as _, *s as _, *l as _, *a as _]),
    }
}

///
/// Used for things like colours where there are four floats in a property list value
///
#[inline]
fn four_floats(vals: &Vec<f32>) -> Option<(f64, f64, f64, f64)> {
    if vals.len() == 4 {
        Some((vals[0] as _, vals[1] as _, vals[2] as _, vals[3] as _))
    } else {
        None
    }
}

///
/// Retrieves a float value from a property if it matches
///
#[inline]
fn float(val: &CanvasProperty) -> Option<f64> {
    match val {
        CanvasProperty::Float(val)  => Some(*val as _),
        _                           => None
    }
}

///
/// Tries to create a colour from canvas properties with the type and value in them
///
pub fn color_from_properties(type_property: &CanvasProperty, value_property: &CanvasProperty) -> Option<Color> {
    use CanvasProperty::*;

    match (type_property, value_property) {
        (Int(0), FloatList(vals)) => { let (r, g, b, a) = four_floats(vals)?; Some(Color::Rgba(r as _, g as _, b as _, a as _)) }
        (Int(1), FloatList(vals)) => { let (h, s, l, a) = four_floats(vals)?; Some(Color::Hsluv(h as _, s as _, l as _, a as _)) }

        _ => None,
    }
}

///
/// Returns the property to use for a linecap value
///
pub fn linecap_property(linecap: &LineCap) -> CanvasProperty {
    match linecap {
        LineCap::Butt   => { CanvasProperty::Int(0) },
        LineCap::Round  => { CanvasProperty::Int(1) },
        LineCap::Square => { CanvasProperty::Int(2) },
    }
}

///
/// Returns the property value to use for a linejoin value
///
pub fn linejoin_property(linejoin: &LineJoin) -> CanvasProperty {
    match linejoin {
        LineJoin::Miter => { CanvasProperty::Int(0) },
        LineJoin::Round => { CanvasProperty::Int(1) },
        LineJoin::Bevel => { CanvasProperty::Int(2) },
    }
}

///
/// Creates a linecap from a canvas property
///
pub fn linecap_from_property(property: &CanvasProperty) -> Option<LineCap> {
    use CanvasProperty::*;

    match property {
        Int(0) => Some(LineCap::Butt),
        Int(1) => Some(LineCap::Round),
        Int(2) => Some(LineCap::Square),

        _ => None
    }
}

///
/// Creates a linejoin from a canvas property
///
pub fn linejoin_from_property(property: &CanvasProperty) -> Option<LineJoin> {
    use CanvasProperty::*;

    match property {
        Int(0) => Some(LineJoin::Miter),
        Int(1) => Some(LineJoin::Round),
        Int(2) => Some(LineJoin::Bevel),

        _ => None
    }
}

impl ToCanvasProperties for FlatFill {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        vec![
            (*PROP_FILL_COLOR_TYPE, color_type_property(&self.0)),
            (*PROP_FILL_COLOR,      color_value_property(&self.0)),
        ]
    }

    fn used_properties() -> Vec<CanvasPropertyId> {
        vec![*PROP_FILL_COLOR_TYPE, *PROP_FILL_COLOR]
    }

    fn from_properties<'a>(properties: impl Iterator<Item=&'a (CanvasPropertyId, CanvasProperty)>) -> Option<Self> {
        let mut fill_color_type  = None;
        let mut fill_color_value = None;

        for (prop_id, prop_val) in properties {
            if *prop_id == *PROP_FILL_COLOR_TYPE { fill_color_type = Some(prop_val); }
            if *prop_id == *PROP_FILL_COLOR      { fill_color_value = Some(prop_val); }
        }

        Some(FlatFill(color_from_properties(fill_color_type?, fill_color_value?)?))
    }
}

impl ToCanvasProperties for Stroke {
    fn to_properties(&self) -> Vec<(CanvasPropertyId, CanvasProperty)> {
        let Stroke(width, cap, join, color) = &self;

        vec![
            (*PROP_STROKE_COLOR_TYPE, color_type_property(color)),
            (*PROP_STROKE_COLOR,      color_value_property(color)),
            (*PROP_STROKE_LINECAP,    linecap_property(cap)),
            (*PROP_STROKE_LINEJOIN,   linejoin_property(join)),
            (*PROP_STROKE_WIDTH,      CanvasProperty::Float(width.0 as _)),
        ]
    }

    fn used_properties() -> Vec<CanvasPropertyId> {
        vec![*PROP_STROKE_COLOR_TYPE, *PROP_STROKE_COLOR, *PROP_STROKE_LINECAP, *PROP_STROKE_LINEJOIN, *PROP_STROKE_WIDTH]
    }

    fn from_properties<'a>(properties: impl Iterator<Item=&'a (CanvasPropertyId, CanvasProperty)>) -> Option<Self> {
        let mut stroke_color_type  = None;
        let mut stroke_color_value = None;
        let mut cap                = None;
        let mut join               = None;
        let mut width              = None;

        for (prop_id, prop_val) in properties {
            if *prop_id == *PROP_STROKE_COLOR_TYPE      { stroke_color_type = Some(prop_val); }
            else if *prop_id == *PROP_STROKE_COLOR      { stroke_color_value = Some(prop_val); }
            else if *prop_id == *PROP_STROKE_LINECAP    { cap = Some(prop_val); }
            else if *prop_id == *PROP_STROKE_LINEJOIN   { join = Some(prop_val); }
            else if *prop_id == *PROP_STROKE_WIDTH      { width = Some(prop_val); }
        }

        Some(Stroke(StrokeWidth(float(width?)?), linecap_from_property(cap?)?, linejoin_from_property(join?)?, color_from_properties(stroke_color_type?, stroke_color_value?)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn round_trip_flatfill_rgba() {
        let original_fill   = FlatFill(Color::Rgba(0.1, 0.2, 0.3, 0.4));
        let properties      = original_fill.to_properties();
        let fill_from_props = FlatFill::from_properties(properties.iter());

        assert!(Some(original_fill) == fill_from_props, "{:?} != {:?}", Some(original_fill), fill_from_props);
    }

    #[test]
    pub fn round_trip_flatfill_hsluv() {
        let original_fill   = FlatFill(Color::Hsluv(0.1, 0.2, 0.3, 0.4));
        let properties      = original_fill.to_properties();
        let fill_from_props = FlatFill::from_properties(properties.iter());

        assert!(Some(original_fill) == fill_from_props, "{:?} != {:?}", Some(original_fill), fill_from_props);
    }

    #[test]
    pub fn round_trip_stroke() {
        let original_stroke     = Stroke(StrokeWidth(42.0), LineCap::Round, LineJoin::Bevel, Color::Rgba(0.4, 0.3, 0.2, 0.1));
        let properties          = original_stroke.to_properties();
        let stroke_from_props   = Stroke::from_properties(properties.iter());

        assert!(Some(original_stroke) == stroke_from_props, "{:?} != {:?}", Some(original_stroke), stroke_from_props);
    }

    #[test]
    pub fn stroke_from_properties() {
        let original_stroke     = Stroke(StrokeWidth(42.0), LineCap::Round, LineJoin::Bevel, Color::Rgba(0.4, 0.3, 0.2, 0.1));
        let properties          = vec![
            (CanvasPropertyId::new("flowbetween::stroke_color_type"),   CanvasProperty::Int(0)), 
            (CanvasPropertyId::new("flowbetween::stroke_color"),        CanvasProperty::FloatList(vec![0.4, 0.3, 0.2, 0.1])), 
            (CanvasPropertyId::new("flowbetween::stroke_linecap"),      CanvasProperty::Int(1)), 
            (CanvasPropertyId::new("flowbetween::stroke_linejoin"),     CanvasProperty::Int(2)), 
            (CanvasPropertyId::new("flowbetween::stroke_width"),        CanvasProperty::Float(42.0)),
        ];
        let stroke_from_props   = Stroke::from_properties(properties.iter());

        assert!(Some(original_stroke) == stroke_from_props, "{:?} != {:?}", Some(original_stroke), stroke_from_props);
    }
}

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
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct FlatFill(pub Color);

///
/// Width of a stroke
///
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct StrokeWidth(pub f64);

///
/// Property applied to a shape that should be surrounded by a line stroke
///
#[derive(Clone, Copy, Serialize, Deserialize)]
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
fn four_floats(vals: &Vec<f64>) -> Option<(f64, f64, f64, f64)> {
    if vals.len() == 4 {
        Some((vals[0], vals[1], vals[2], vals[3]))
    } else {
        None
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

    fn from_properties<'a>(&'a self, properties: impl Iterator<Item=&'a (CanvasPropertyId, CanvasProperty)>) -> Option<Self> {
        let mut fill_color_type  = None;
        let mut fill_color_value = None;

        for (prop_id, prop_val) in properties {
            if *prop_id == *PROP_FILL_COLOR_TYPE { fill_color_type = Some(prop_val); }
            if *prop_id == *PROP_FILL_COLOR      { fill_color_value = Some(prop_val); }
        }

        Some(FlatFill(color_from_properties(fill_color_type?, fill_color_value?)?))
    }
}

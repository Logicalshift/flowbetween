use super::geometry::*;
use super::region_description::*;
use super::effect_description::*;
use crate::region::*;
use crate::effects::*;

use flo_curves::*;
use flo_curves::bezier::path::{SimpleBezierPath};

use serde_json as json;

use std::sync::*;
use std::collections::{HashMap};

lazy_static! {
    pub (super) static ref OTHER_EFFECTS: Mutex<HashMap<String, Box<dyn Send+Sync+Fn(&json::Value) -> Box<dyn AnimationEffect>>>> = Mutex::new(HashMap::new());
}

impl From<Coord2> for Point2D {
    #[inline]
    fn from(Coord2(x, y): Coord2) -> Point2D {
        Point2D(x, y)
    }
}

impl From<&Coord2> for Point2D {
    #[inline]
    fn from(Coord2(x, y): &Coord2) -> Point2D {
        Point2D(*x, *y)
    }
}

impl Into<Coord2> for Point2D {
    #[inline]
    fn into(self) -> Coord2 {
        let Point2D(x, y) = self;
        Coord2(x, y)
    }
}

impl Into<Coord2> for &Point2D {
    #[inline]
    fn into(self) -> Coord2 {
        let Point2D(x, y) = self;
        Coord2(*x, *y)
    }
}

impl From<SimpleBezierPath> for BezierPath {
    fn from((start_point, coords): SimpleBezierPath) -> BezierPath {
        BezierPath(start_point.into(), coords.into_iter().map(|(cp1, cp2, end_point)| BezierPoint(cp1.into(), cp2.into(), end_point.into())).collect())
    }
}

impl Into<SimpleBezierPath> for BezierPath {
    fn into(self) -> SimpleBezierPath {
        let BezierPath(start_point, coords) = self;

        (start_point.into(), coords.into_iter().map(|BezierPoint(cp1, cp2, end_point)| (cp1.into(), cp2.into(), end_point.into())).collect())
    }
}

impl From<&SimpleBezierPath> for BezierPath {
    fn from((start_point, coords): &SimpleBezierPath) -> BezierPath {
        BezierPath((*start_point).into(), coords.into_iter().map(|(cp1, cp2, end_point)| BezierPoint(cp1.into(), cp2.into(), end_point.into())).collect())
    }
}

impl Into<SimpleBezierPath> for &BezierPath {
    fn into(self) -> SimpleBezierPath {
        let BezierPath(start_point, coords) = self;

        ((*start_point).into(), coords.iter().map(|BezierPoint(cp1, cp2, end_point)| (cp1.into(), cp2.into(), (*end_point).into())).collect())
    }
}

impl Into<Box<dyn AnimationEffect>> for EffectDescription {
    #[inline]
    fn into(self) -> Box<dyn AnimationEffect> {
        (&self).into()
    }
}

impl Into<Box<dyn AnimationEffect>> for &EffectDescription {
    fn into(self) -> Box<dyn AnimationEffect> {
        use self::EffectDescription::*;

        match self {
            Other(type_name, json_defn)                 => OTHER_EFFECTS.lock().unwrap().get(type_name)
                                                                .unwrap_or_else(|| panic!("Animation effect '{}' is not registered", type_name))(json_defn),
            Sequence(sequence)                          => unimplemented!(),

            Repeat(time, effect)                        => Box::new(RepeatEffect::<Box<dyn AnimationEffect>>::repeat_effect((&**effect).into(), *time)),
            TimeCurve(curve_points, effect)             => Box::new(TimeCurveEffect::<Box<dyn AnimationEffect>>::with_control_points((&**effect).into(), curve_points.clone())),

            Move(time, BezierPath(start_point, coords)) => Box::new(MotionEffect::from_points(*time, start_point.into(), coords.iter().map(|BezierPoint(cp1, cp2, ep)| (cp1.into(), cp2.into(), ep.into())).collect()))
        }
    }
}

impl Into<Arc<dyn AnimationRegion>> for &RegionDescription {
    fn into(self) -> Arc<dyn AnimationRegion> {
        let RegionDescription(path, effect)     = self;
        let effect: Box<dyn AnimationEffect>    = effect.into();
        let path: Vec<SimpleBezierPath>         = path.iter().map(|path| path.into()).collect();

        Arc::new(effect.with_region(path))
    }
}

///
/// Adds a custom 'other' effect deserializer
///
/// This makes it possible to add effect descriptions for effects that aren't in the `flo_canvas_animation` library itself, by adding
/// new serializers and using the `EffectDescription::Other()` description. Type names should be unique, so something using domain names
/// is a good idea, eg `app.flowbetween.CustomEffect`.
///
pub fn register_animation_effect_deserializer<TFn: 'static+Send+Sync+Fn(&json::Value) -> Box<dyn AnimationEffect>>(type_name: &str, deserialization_fn: TFn) {
    let old_key = OTHER_EFFECTS.lock().unwrap()
        .insert(type_name.into(), Box::new(deserialization_fn));

    if old_key.is_some() {
        panic!("Animation effect '{}' was registered more than once", type_name);
    }
}

///
/// Indicates if a particular animation effect type name is available to deserialize
///
/// (Type names can be registered but not deregistered, so if this returns true, the type name will be available for as long as the
/// program is running)
///
pub fn is_animation_effect_type_registered(type_name: &str) -> bool {
    OTHER_EFFECTS.lock().unwrap()
        .contains_key(type_name)
}

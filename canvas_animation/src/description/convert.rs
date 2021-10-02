use super::geometry::*;
use super::region_description::*;
use super::effect_description::*;
use crate::region::*;
use crate::effects::*;

use flo_curves::*;
use flo_curves::bezier::path::{SimpleBezierPath};

use std::sync::*;

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
            Other(type_name, json_defn)                 => unimplemented!(),
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

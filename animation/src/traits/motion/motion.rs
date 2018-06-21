use super::translate::*;
use super::transform::*;
use super::motion_type::*;
use super::super::brush::*;
use super::super::time_path::*;

use std::ops::Range;
use std::time::Duration;

///
/// Describes ways in which a vector element can be moved and transformed over time.
/// Every element can have more than one motion attached to it, but for any given
/// element, each motion must appear only once.
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum Motion {
    /// Motion with no effect
    None,

    /// Describes how an element is translated over time
    Translate(TranslateMotion)
}

impl Motion {
    ///
    /// Retrieves the type of this motion
    /// 
    pub fn motion_type(&self) -> MotionType {
        use self::Motion::*;

        match self {
            None            => MotionType::None,
            Translate(_)    => MotionType::Translate
        }
    }

    ///
    /// Sets this motion to be a particular type
    /// 
    pub fn set_type(&mut self, motion_type: MotionType) {
        use self::MotionType::*;

        match motion_type {
            None        => { *self = Motion::None; },
            Translate   => { *self = Motion::Translate(TranslateMotion::default()); }
        }
    }

    ///
    /// Sets the origin of this motion
    /// 
    pub fn set_origin(&mut self, new_origin: (f32, f32)) {
        use self::Motion::*;

        match self {
            None                    => { }
            Translate(translate)    => { translate.set_origin(new_origin); }
        }
    }

    ///
    /// Sets the path of this motion
    /// 
    pub fn set_path(&mut self, new_path: TimeCurve) {
        use self::Motion::*;

        match self {
            None                    => { }
            Translate(translate)    => { translate.set_path(new_path); }
        }
    }
}

impl MotionTransform for Motion {
    fn range_millis(&self) -> Range<f32> {
        use self::Motion::*;

        match self {
            None                    => 0.0..0.0,
            Translate(translate)    => translate.range_millis()
        }
    }

    fn transform_points<'a, Points: 'a+Iterator<Item=&'a BrushPoint>>(&self, time: Duration, points: Points) -> Box<dyn 'a+Iterator<Item=BrushPoint>> {
        use self::Motion::*;

        match self {
            None                    => Box::new(points.cloned()),
            Translate(translate)    => translate.transform_points(time, points)
        }
    }
}

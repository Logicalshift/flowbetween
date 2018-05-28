use super::translate::*;
use super::transform::*;
use super::super::raw_point::*;

use std::ops::Range;
use std::time::Duration;

///
/// Describes ways in which a vector element can be moved and transformed over time.
/// Every element can have more than one motion attached to it, but for any given
/// element, each motion must appear only once.
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum Motion {
    /// Describes how an element is translated over time
    Translate(TranslateMotion)
}

impl MotionTransform for Motion {
    fn range_millis(&self) -> Range<f32> {
        use self::Motion::*;

        match self {
            Translate(translate)    => translate.range_millis()
        }
    }

    fn transform_points<'a, Points: 'a+Iterator<Item=RawPoint>>(&self, time: Duration, points: Points) -> Box<'a+Iterator<Item=RawPoint>> {
        use self::Motion::*;

        match self {
            Translate(translate)    => translate.transform_points(time, points)
        }
    }
}

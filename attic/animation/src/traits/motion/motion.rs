use super::translate::*;
use super::transform::*;
use super::motion_type::*;
use super::super::vector::*;
use super::super::time_path::*;

use smallvec::*;

use std::sync::*;
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

    /// Performs the reverse of the specified motion
    Reverse(Arc<Motion>),

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
            Reverse(_)      => MotionType::Reverse,
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
            Reverse     => { *self = Motion::Reverse(Arc::new(Motion::None)); }
            Translate   => { *self = Motion::Translate(TranslateMotion::default()); }
        }
    }

    ///
    /// Sets the origin of this motion
    ///
    pub fn set_origin(&mut self, new_origin: (f32, f32)) {
        use self::Motion::*;

        match self {
            None                    => { },
            Reverse(_)              => { },
            Translate(translate)    => { translate.set_origin(new_origin); }
        }
    }

    ///
    /// Sets the path of this motion
    ///
    pub fn set_path(&mut self, new_path: TimeCurve) {
        use self::Motion::*;

        match self {
            None                    => { },
            Reverse(_)              => { },
            Translate(translate)    => { translate.set_path(new_path); }
        }
    }

    ///
    /// Changes this to the reverse motion of itself
    ///
    pub fn reverse(self) -> Motion {
        match self {
            Motion::Reverse(reversed)   => (&*reversed).clone(),
            motion                      => Motion::Reverse(Arc::new(motion))
        }
    }
}

impl MotionTransform for Motion {
    fn range_millis(&self) -> Range<f32> {
        use self::Motion::*;

        match self {
            None                    => 0.0..0.0,
            Reverse(motion)         => motion.range_millis(),
            Translate(translate)    => translate.range_millis()
        }
    }

    ///
    /// Returns the transformations to apply for this motion at a particular point in time
    ///
    fn transformation(&self, when: Duration) -> SmallVec<[Transformation; 2]> {
        use self::Motion::*;

        match self {
            None                    => smallvec![],
            Translate(translate)    => translate.transformation(when),

            Reverse(motion)         => {
                let transform = motion.transformation(when);
                transform.into_iter()
                    .rev()
                    .flat_map(|transform| transform.invert())
                    .collect()
            }
        }
    }
}

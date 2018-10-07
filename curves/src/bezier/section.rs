use super::curve::*;
use super::basis::*;
use super::super::geo::*;

use std::cell::*;

///
/// Represents a subsection of a bezier curve
/// 
#[derive(Clone)]
pub struct CurveSection<'a, C: 'a+BezierCurve> {
    /// Full curve
    curve: &'a C,

    /// Value to add to a t value to convert to this curve
    t_c: f64,

    /// Value to multiply a t value by to convert to this curve
    t_m: f64,

    /// Cached version of the control points for this curve section
    cached_control_points: RefCell<Option<(C::Point, C::Point)>>
}

impl<'a, C: 'a+BezierCurve> CurveSection<'a, C> {
    ///
    /// Creates a new curve section from a region of another bezier curve
    /// 
    pub fn new(curve: &'a C, t_min: f64, t_max: f64) -> CurveSection<'a, C> {
        let t_c = t_min;
        let t_m = t_max - t_c;

        CurveSection {
            curve:                  curve,
            t_m:                    t_m,
            t_c:                    t_c,
            cached_control_points:  RefCell::new(None)
        }
    }

    ///
    /// Returns the t value on the full curve for a t value on the section
    /// 
    #[inline]
    fn t_for_t(&self, t: f64) -> f64 {
        t*self.t_m + self.t_c
    }

    ///
    /// Returns true if this section is so small as to represent a point
    ///
    #[inline]
    pub fn is_tiny(&self) -> bool {
        let t_min = self.t_c;
        let t_max = self.t_for_t(1.0);

        (t_max-t_min).abs() < 0.000001
    }

    ///
    /// Creates a sub-section from this curve section (dividing it further)
    /// 
    /// Unlike calling `section`, this keeps the same type and avoids the need
    /// for recursive recalculation for things like the control points. This means
    /// that `original_curve_t_values` will return the coordinates for the same
    /// original curve as the curve that this subsection was created from.
    /// 
    pub fn subsection(&self, t_min: f64, t_max: f64) -> CurveSection<'a, C> {
        CurveSection::new(self.curve, self.t_for_t(t_min), self.t_for_t(t_max))
    }

    ///
    /// Returns the original t values (t_min, t_max) that this section was created from
    /// 
    #[inline]
    pub fn original_curve_t_values(&self) -> (f64, f64) {
        (self.t_c, self.t_m+self.t_c)
    }

    ///
    /// Given a 't' value on the original curve, returns the equivalent value on this section
    /// 
    #[inline]
    pub fn section_t_for_original_t(&self, t: f64) -> f64 {
        (t-self.t_c)/self.t_m
    }
}

impl<'a, C: 'a+BezierCurve> Geo for CurveSection<'a, C> {
    type Point = C::Point;
}

impl<'a, C: 'a+BezierCurve> BezierCurve for CurveSection<'a, C> {
    ///
    /// The start point of this curve
    /// 
    #[inline]
    fn start_point(&self) -> Self::Point {
        self.curve.point_at_pos(self.t_for_t(0.0))
    }

    ///
    /// The end point of this curve
    /// 
    #[inline]
    fn end_point(&self) -> Self::Point {
        self.curve.point_at_pos(self.t_for_t(1.0))
    }

    ///
    /// The control points in this curve
    /// 
    fn control_points(&self) -> (Self::Point, Self::Point) {
        self.cached_control_points.borrow_mut()
            .get_or_insert_with(move || {
                // This is the de-casteljau subdivision algorithm (ran twice to cut out a section of the curve)
                let t_min = self.t_c;

                // Get the weights from the curve
                let w1          = self.curve.start_point();
                let (w2, w3)    = self.curve.control_points();
                let w4          = self.curve.end_point();

                // Weights (from de casteljau)
                let wn1 = w1*(1.0-t_min) + w2*t_min;
                let wn2 = w2*(1.0-t_min) + w3*t_min;
                let wn3 = w3*(1.0-t_min) + w4*t_min;

                // Further refine the weights
                let wnn1 = wn1*(1.0-t_min) + wn2*t_min;
                let wnn2 = wn2*(1.0-t_min) + wn3*t_min;

                // Get the point at which the two curves join
                let p = de_casteljau2(t_min, wnn1, wnn2);
                
                // Curve from t_min to 1 is in (p, wnn2, wn3, w4), we need to subdivide again
                let (w1, w2, w3)    = (p, wnn2, wn3);
                let t_max           = self.t_m/(1.0-self.t_c);

                // Weights (from de casteljau)
                let wn1 = w1*(1.0-t_max) + w2*t_max;
                let wn2 = w2*(1.0-t_max) + w3*t_max;
                // let wn3 = w3*(1.0-t_max) + w4*t_max;

                // Further refine the weights
                let wnn1 = wn1*(1.0-t_max) + wn2*t_max;
                // let wnn2 = wn2*(1.0-t_max) + wn3*t_max;
                // let p = de_casteljau2(t_min, wnn1, wnn2);

                // Curve is (w1, wn1, wnn1, p) so control points are wn1 and wnn1
                (wn1, wnn1)
            })
            .clone()
    }

    ///
    /// Given a value t from 0 to 1, returns a point on this curve
    /// 
    #[inline]
    fn point_at_pos(&self, t: f64) -> Self::Point {
        self.curve.point_at_pos(self.t_for_t(t))
    }
}

///
/// Trait implemented by curves that can have sections taken from them
/// 
pub trait BezierCurveWithSections {
    /// The type of a section of this curve
    type SectionCurve: BezierCurve;

    ///
    /// Create a section from this curve. Consider calling `subsection` for curves
    /// that are already `CurveSections`.
    /// 
    fn section(self, t_min: f64, t_max: f64) -> Self::SectionCurve;
}

impl<'a, C: 'a+BezierCurve> BezierCurveWithSections for &'a C {
    type SectionCurve = CurveSection<'a, C>;

    fn section(self, t_min: f64, t_max: f64) -> CurveSection<'a, C> {
        CurveSection::new(self, t_min, t_max)
    }
}

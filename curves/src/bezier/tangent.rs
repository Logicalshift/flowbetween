use super::curve::*;
use super::basis::*;
use super::derivative::*;

///
/// A structure that can be used to compute the tangent of a bezier curve
/// 
pub struct Tangent<Curve: BezierCurve> {
    /// The derivative of the curve
    derivative: (Curve::Point, Curve::Point, Curve::Point)
}

impl<'a, Curve: BezierCurve> From<&'a Curve> for Tangent<Curve> {
    ///
    /// Creates a structure that can computes the tangents for a bezier curve
    /// 
    fn from(curve: &'a Curve) -> Tangent<Curve> {
        let control_points = curve.control_points();

        Tangent {
            derivative: derivative4(curve.start_point(), control_points.0, control_points.1, curve.end_point())
        }
    }
}

impl<Curve: BezierCurve> Tangent<Curve> {
    ///
    /// Calculates the tangent at a particular point
    /// 
    pub fn tangent(&self, t: f64) -> Curve::Point {
        de_casteljau3(t, self.derivative.0, self.derivative.1, self.derivative.2)
    }
}

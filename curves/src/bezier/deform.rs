use super::curve::*;

///
/// Moves the point at 't' on the curve by the offset vector
/// 
/// This recomputes the control points such that the point at t on the original curve
/// is moved by the vector specified by `offset`.
/// 
pub fn move_point<Curve: BezierCurve>(curve: &Curve, t: f64, offset: Curve::Point) -> Curve {
    // Fetch the points from the curve
    let w1          = curve.start_point();
    let w4          = curve.end_point();
    let (w2, w3)    = curve.control_points();

    let one_minus_t         = 1.0-t;
    let one_minus_t_cubed   = one_minus_t*one_minus_t*one_minus_t;
    let t_cubed             = t*t*t;

    // Point 'C' is fixed for the transformation and is along the line w1-w4
    let u = one_minus_t_cubed / (t_cubed + one_minus_t_cubed);
    let c = w1*u + w4*(1.0-u);

    // Construct the de Casteljau points for the point we're moving
    let wn1 = w1*(1.0-t) + w2*t;
    let wn2 = w2*(1.0-t) + w3*t;
    let wn3 = w3*(1.0-t) + w4*t;

    let wnn1 = wn1*(1.0-t) + wn2*t;
    let wnn2 = wn2*(1.0-t) + wn3*t;

    let p = wnn1*(1.0-t) + wnn2*t;

    // Translating wnn1 and wnn2 by the offset will give us a new p that is also translated by the offset
    let pb = p + offset;

    let wnn1b = wnn1 + offset;
    let wnn2b = wnn2 + offset;

    // The line c->pb->wn2b has the same ratios as the line c->p->wn2, so we can compute wn2b
    // There's a trick to calculating this for cubic curves (which is handy as it means this will work with straight lines as well as curves)
    let ratio   = ((t_cubed+one_minus_t_cubed)/(t_cubed + one_minus_t_cubed-1.0)).abs();
    let wn2b    = ((pb-c)*ratio) + pb;

    // We can now calculate wn1b and wn3b
    let inverse_t       = 1.0/t;
    let inverse_tminus1 = 1.0/(t-1.0);
    
    let wn1b = (wn2b*t - wnn1b)*inverse_tminus1;
    let wn3b = (wn2b*-1.0 + wn2b*t + wnn2b)*inverse_t;

    // ... and the new control points
    let w2b = (w1*-1.0 + w1*t + wn1b)*inverse_t;
    let w3b = (w4*t-wn3b)*inverse_tminus1;

    // Use the values to construct the curve with the moved point
    Curve::from_points(w1, w4, w2b, w3b)
}
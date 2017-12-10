use super::basis::*;

///
/// Subdivides a cubic bezier curve at a particular point, returning the weights of
/// the two component curves
/// 
pub fn subdivide4(t: f32, w1: f32, w2: f32, w3: f32, w4: f32) -> 
    ((f32, f32, f32, f32),
    (f32, f32, f32, f32)) {
    // Weights (from de casteljau)
    let wn1 = (1.0-t)*w1 + t*w2;
    let wn2 = (1.0-t)*w2 + t*w3;
    let wn3 = (1.0-t)*w3 + t*w4;

    // Further refine the weights
    let wnn1 = (1.0-t)*wn1 + t*wn2;
    let wnn2 = (1.0-t)*wn2 + t*wn3;

    // Get the point at which the two curves join
    let p = de_casteljau2(t, wnn1, wnn2);

    // Curves are built from the weight calculations and the final points
    ((w1, wn1, wnn1, p), (p, wnn2, wn3, w4))
}

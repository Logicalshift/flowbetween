///
/// The cubic bezier weighted basis function
/// 
#[inline]
pub fn basis(t: f32, w1: f32, w2: f32, w3: f32, w4: f32) -> f32 {
    let t_squared           = t*t;
    let t_cubed             = t_squared*t;

    let one_minus_t         = 1.0-t;
    let one_minus_t_squared = one_minus_t*one_minus_t;
    let one_minus_t_cubed   = one_minus_t_squared*one_minus_t;

    return w1*one_minus_t_cubed 
        + 3.0*w2*one_minus_t_squared*t
        + 3.0*w3*one_minus_t*t_squared
        + w4*t_cubed;
}


///
/// de Casteljau's algorithm for cubic bezier curves
/// 
#[inline]
pub fn de_casteljau4(t: f32, w1: f32, w2: f32, w3: f32, w4: f32) -> f32 {
    let wn1 = (1.0-t)*w1 + t*w2;
    let wn2 = (1.0-t)*w2 + t*w3;
    let wn3 = (1.0-t)*w3 + t*w4;

    de_casteljau3(t, wn1, wn2, wn3)
}

///
/// de Casteljau's algorithm for quadratic bezier curves
/// 
#[inline]
pub fn de_casteljau3(t: f32, w1: f32, w2: f32, w3: f32) -> f32 {
    let wn1 = (1.0-t)*w1 + t*w2;
    let wn2 = (1.0-t)*w2 + t*w3;

    de_casteljau2(t, wn1, wn2)
}

///
/// de Casteljau's algorithm for lines
/// 
#[inline]
pub fn de_casteljau2(t: f32, w1: f32, w2: f32) -> f32 {
    (1.0-t)*w1 + t*w2
}

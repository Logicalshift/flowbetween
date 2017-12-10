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

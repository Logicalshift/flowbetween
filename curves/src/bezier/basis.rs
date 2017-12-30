use super::super::coordinate::*;

///
/// The cubic bezier weighted basis function
/// 
#[inline]
pub fn basis<Point: Coordinate>(t: f64, w1: Point, w2: Point, w3: Point, w4: Point) -> Point {
    let t_squared           = t*t;
    let t_cubed             = t_squared*t;

    let one_minus_t         = 1.0-t;
    let one_minus_t_squared = one_minus_t*one_minus_t;
    let one_minus_t_cubed   = one_minus_t_squared*one_minus_t;

    w1*one_minus_t_cubed 
        + w2*3.0*one_minus_t_squared*t
        + w3*3.0*one_minus_t*t_squared
        + w4*t_cubed
}


///
/// de Casteljau's algorithm for cubic bezier curves
/// 
#[inline]
pub fn de_casteljau4<Point: Coordinate>(t: f64, w1: Point, w2: Point, w3: Point, w4: Point) -> Point {
    let wn1 = w1*(1.0-t) + w2*t;
    let wn2 = w2*(1.0-t) + w3*t;
    let wn3 = w3*(1.0-t) + w4*t;

    de_casteljau3(t, wn1, wn2, wn3)
}

///
/// de Casteljau's algorithm for quadratic bezier curves
/// 
#[inline]
pub fn de_casteljau3<Point: Coordinate>(t: f64, w1: Point, w2: Point, w3: Point) -> Point {
    let wn1 = w1*(1.0-t) + w2*t;
    let wn2 = w2*(1.0-t) + w3*t;

    de_casteljau2(t, wn1, wn2)
}

///
/// de Casteljau's algorithm for lines
/// 
#[inline]
pub fn de_casteljau2<Point: Coordinate>(t: f64, w1: Point, w2: Point) -> Point {
    w1*(1.0-t) + w2*t
}

use flo_curves::*;
use flo_canvas_animation::description::*;

use std::f64;

#[test]
fn rotate_radians_to_degrees() {
    let radians                 = RotateRadians(f64::consts::PI);
    let degrees: RotateDegrees  = radians.into();

    assert!(degrees == RotateDegrees(180.0));
}

#[test]
fn rotate_degrees_to_radians() {
    let degrees                 = RotateDegrees(180.0);
    let radians: RotateRadians  = degrees.into();

    assert!(radians == RotateRadians(f64::consts::PI));
}

#[test]
fn translate_with_anchor() {
    let point           = Point2D(10.0, 10.0);
    let transform       = TransformWithAnchor(Point2D(5.0, 5.0), TransformPoint(Point2D(1.0, 5.0), Scale::default(), RotateRadians::default()));
    let transform_point = point.transform(&transform.into());

    assert!(transform_point.distance_to(&Point2D(11.0, 15.0)) < 0.1);
}

#[test]
fn scale_with_anchor() {
    let point           = Point2D(10.0, 10.0);
    let transform       = TransformWithAnchor(Point2D(5.0, 5.0), TransformPoint(Point2D::default(), Scale(2.0, 2.0), RotateRadians::default()));
    let transform_point = point.transform(&transform.into());

    println!("{:?}", transform_point);
    assert!(transform_point.distance_to(&Point2D(15.0, 15.0)) < 0.1);
}

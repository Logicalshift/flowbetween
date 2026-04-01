use flo_canvas_animation::description::*;

use serde_json as json;

use std::time::{Duration};

#[test]
fn move_description_round_trip() {
    let move_description    = EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))]));
    let as_json             = json::to_string(&move_description).unwrap();
    let description_again   = json::from_str(&as_json).unwrap();

    assert!(move_description == description_again);
}

#[test]
fn deserialize_move_description() {
    let move_description    = EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))]));
    let as_json             = "{\"Move\":[{\"secs\":10,\"nanos\":0},[[20.0,30.0],[[[20.0,100.0],[200.0,200.0],[300.0,400.0]]]]]}";
    let description_again   = json::from_str(&as_json).unwrap();

    assert!(move_description == description_again);
}

#[test]
fn deserialize_fitted_transform_description() {
    let transform_description   = EffectDescription::FittedTransform(
        Point2D(500.0, 500.0),
        vec![
            TransformPoint(Point2D(0.0, 0.0), Scale::default(), RotateRadians::default()).with_time(Duration::from_secs(0)),
            TransformPoint(Point2D(0.0, 0.0), Scale::default(), RotateRadians::default()).with_time(Duration::from_millis(100)),
            TransformPoint(Point2D(42.0, 43.0), Scale(1.5, 1.5), RotateDegrees(180.0).into()).with_time(Duration::from_secs(6)),
            TransformPoint(Point2D(0.0, 0.0), Scale::default(), RotateDegrees(360.0).into()).with_time(Duration::from_secs(10)),
            TransformPoint(Point2D(42.0*2.0, 43.0*2.0), Scale(0.25, 0.25), RotateDegrees(360.0 + 180.0).into()).with_time(Duration::from_secs(13)),
            TransformPoint(Point2D(42.0*2.0, 43.0*2.0), Scale(1.0, 1.0), RotateDegrees(360.0 + 270.0).into()).with_time(Duration::from_secs(17)),
            TransformPoint(Point2D(0.0, 0.0), Scale::default(), RotateDegrees(360.0 + 360.0).into()).with_time(Duration::from_millis(19_900)),
            TransformPoint(Point2D(0.0, 0.0), Scale::default(), RotateDegrees(360.0 + 360.0).into()).with_time(Duration::from_secs(20)),
        ]
    );
    let as_json                 = "{\"FittedTransform\":[[500.0,500.0],[[[[0.0,0.0],[1.0,1.0],0.0],0.0],[[[0.0,0.0],[1.0,1.0],0.0],100.0],[[[42.0,43.0],[1.5,1.5],3.141592653589793],6000.0],[[[0.0,0.0],[1.0,1.0],6.283185307179586],10000.0],[[[84.0,86.0],[0.25,0.25],9.42477796076938],13000.0],[[[84.0,86.0],[1.0,1.0],10.995574287564276],17000.0],[[[0.0,0.0],[1.0,1.0],12.566370614359172],19900.0],[[[0.0,0.0],[1.0,1.0],12.566370614359172],20000.0]]]}";
    let description_again       = json::from_str(&as_json).unwrap();

    assert!(transform_description == description_again);
}

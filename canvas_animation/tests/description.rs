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

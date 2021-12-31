use flo_canvas_animation::description::*;

use std::time::{Duration};

#[test]
fn empty_sequence() {
    let effect = EffectDescription::Sequence(vec![]);

    assert!(effect.sub_effects().len() == 0);
}

#[test]
fn move_effect() {
    let effect = EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))]));

    assert!(effect.sub_effects().len() == 1);
    assert!(effect.sub_effects()[0].effect_type() == SubEffectType::LinearPosition);
}

#[test]
fn fitted_transform() {
    let effect = EffectDescription::FittedTransform(
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

    assert!(effect.sub_effects().len() == 1);
    assert!(effect.sub_effects()[0].effect_type() == SubEffectType::TransformPosition);
}

#[test]
fn stop_motion_transform() {
    let effect = EffectDescription::StopMotionTransform(
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

    assert!(effect.sub_effects().len() == 1);
    assert!(effect.sub_effects()[0].effect_type() == SubEffectType::TransformPosition);
}

#[test]
fn two_element_sequence() {
    // Combining these effects probably doesn't make a lot of sense
    let effect = EffectDescription::Sequence(vec![
        EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))])),
        EffectDescription::StopMotionTransform(
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
        ),
    ]);

    assert!(effect.sub_effects().len() == 2);
    assert!(effect.sub_effects()[0].effect_type() == SubEffectType::LinearPosition);
    assert!(effect.sub_effects()[1].effect_type() == SubEffectType::TransformPosition);
}

#[test]
fn top_level_repeat() {
    let effect = EffectDescription::Repeat(Duration::from_millis(1000),
        EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))])).boxed());

    assert!(effect.sub_effects().len() == 2);
    assert!(effect.sub_effects()[0].effect_type() == SubEffectType::Repeat);
    assert!(effect.sub_effects()[1].effect_type() == SubEffectType::LinearPosition);
}

#[test]
fn top_level_timecurve() {
    let effect = EffectDescription::TimeCurve(vec![],
        EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))])).boxed());

    assert!(effect.sub_effects().len() == 2);
    assert!(effect.sub_effects()[0].effect_type() == SubEffectType::TimeCurve);
    assert!(effect.sub_effects()[1].effect_type() == SubEffectType::LinearPosition);
}

#[test]
fn replace_sub_effect_sequence() {
    let effect = EffectDescription::Sequence(vec![
        EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))])),
        EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))])),
        EffectDescription::Move(Duration::from_millis(10000), BezierPath(Point2D(20.0, 30.0), vec![BezierPoint(Point2D(20.0, 100.0), Point2D(200.0, 200.0), Point2D(300.0, 400.0))])),
    ]);

    assert!(effect.sub_effects().len() == 3);

    let new_effect = effect.replace_sub_effect(&effect.sub_effects()[1], EffectDescription::StopMotionTransform(Point2D(500.0, 500.0), vec![]));
    assert!(new_effect.sub_effects().len() == 3);
    assert!(new_effect.sub_effects()[1].effect_type() == SubEffectType::TransformPosition);

    assert!(new_effect.sub_effects()[0].effect_type() == SubEffectType::LinearPosition);
    assert!(new_effect.sub_effects()[2].effect_type() == SubEffectType::LinearPosition);
}

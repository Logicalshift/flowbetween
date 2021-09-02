use flo_curves::*;
use flo_canvas_animation::effects::*;

#[test]
pub fn basic_spacing() {
    let effect              = MotionEffect::from_points(10000.0, Coord2(20.0, 30.0), vec![(Coord2(20.0, 100.0), Coord2(200.0, 200.0), Coord2(300.0, 400.0))]);

    let initial_offset      = effect.offset_at_time(0.0, 0.01);
    assert!(initial_offset.magnitude() <= 0.1);

    let mut last_offset     = initial_offset;
    let mut time            = 1000.0/60.0;
    let mut last_distance   = Option::<f64>::None;
    while time <= 10000.0 {
        let offset          = effect.offset_at_time(time, 0.01);
        let distance        = last_offset.distance_to(&offset);

        println!("{:?}", distance);

        if let Some(last_distance) = last_distance {
            assert!((distance-last_distance).abs() < 0.1);
        }

        last_offset         = offset;
        last_distance       = Some(distance);

        // 60 fps
        time += 1000.0 / 60.0;
    }
}

#[test]
pub fn two_segment_spacing() {
    let effect              = MotionEffect::from_points(10000.0, Coord2(20.0, 30.0), vec![(Coord2(20.0, 100.0), Coord2(200.0, 200.0), Coord2(300.0, 400.0)), (Coord2(500.0, 400.0), Coord2(200.0, 100.0), Coord2(100.0, 100.0))]);

    let initial_offset      = effect.offset_at_time(0.0, 0.01);
    assert!(initial_offset.magnitude() <= 0.1);

    let mut last_offset     = initial_offset;
    let mut time            = 1000.0/60.0;
    let mut last_distance   = Option::<f64>::None;
    while time <= 10000.0 {
        let offset          = effect.offset_at_time(time, 0.01);
        let distance        = last_offset.distance_to(&offset);

        println!("{:?}", distance);

        if let Some(last_distance) = last_distance {
            assert!((distance-last_distance).abs() < 0.1);
        }

        last_offset         = offset;
        last_distance       = Some(distance);

        // 60 fps
        time += 1000.0 / 60.0;
    }
}

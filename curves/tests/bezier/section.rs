use flo_curves::*;
use flo_curves::bezier::*;

#[test]
fn section_points_match() {
    let original_curve  = Curve::from_points(Coord2(2.0, 3.0), Coord2(6.0, 2.0), Coord2(4.0, 5.0), Coord2(5.0, 0.0));
    let mid_section     = original_curve.section(0.25, 0.75);

    for t in 0..=10 {
        let t   = (t as f64)/10.0;
        let t2  = t*0.5 + 0.25;

        let p1 = mid_section.point_at_pos(t);
        let p2 = original_curve.point_at_pos(t2);

        assert!(p1.distance_to(&p2) < 0.0001);
    }
}

#[test]
fn generate_curve_from_section() {
    let original_curve  = Curve::from_points(Coord2(2.0, 3.0), Coord2(6.0, 2.0), Coord2(4.0, 5.0), Coord2(5.0, 0.0));
    let mid_section     = Curve::from_curve(&original_curve.section(0.2, 0.6));

    for t in 0..=10 {
        let t   = (t as f64)/10.0;
        let t2  = t*0.4 + 0.2;

        let p1 = mid_section.point_at_pos(t);
        let p2 = original_curve.point_at_pos(t2);

        assert!(p1.distance_to(&p2) < 0.0001);
    }
}

#[test]
fn section_of_section() {
    let original_curve  = Curve::from_points(Coord2(2.0, 3.0), Coord2(6.0, 2.0), Coord2(4.0, 5.0), Coord2(5.0, 0.0));
    let mut mid_section = original_curve.section(0.25, 0.75);
    mid_section         = mid_section.subsection(0.25, 0.75);

    for t in 0..=10 {
        let t   = (t as f64)/10.0;
        let t2  = t*0.25 + 0.375;

        let p1 = mid_section.point_at_pos(t);
        let p2 = original_curve.point_at_pos(t2);

        assert!(p1.distance_to(&p2) < 0.0001);
    }
}

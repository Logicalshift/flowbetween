///
/// Serialization tests for canvas shape types
///

use crate::scenery::document::canvas::point::*;
use crate::scenery::document::canvas::shape::*;

/// Helper to create a test CanvasPoint
fn test_point(x: f32, y: f32) -> CanvasPoint {
    CanvasPoint { x, y }
}

// ============================================
// Round-trip tests
// ============================================

#[test]
fn ellipse_round_trip() {
    let ellipse = CanvasEllipse {
        min:       test_point(10.0, 20.0),
        max:       test_point(100.0, 200.0),
        direction: test_point(1.0, 0.0),
    };

    let serialized = postcard::to_allocvec(&ellipse).expect("Failed to serialize ellipse");
    let deserialized: CanvasEllipse =
        postcard::from_bytes(&serialized).expect("Failed to deserialize ellipse");

    assert_eq!(ellipse, deserialized);
}

#[test]
fn rectangle_round_trip() {
    let rectangle = CanvasRectangle {
        min: test_point(5.0, 15.0),
        max: test_point(50.0, 150.0),
    };

    let serialized = postcard::to_allocvec(&rectangle).expect("Failed to serialize rectangle");
    let deserialized: CanvasRectangle =
        postcard::from_bytes(&serialized).expect("Failed to deserialize rectangle");

    assert_eq!(rectangle, deserialized);
}

#[test]
fn polygon_round_trip() {
    let polygon = CanvasPolygon {
        min:       test_point(0.0, 0.0),
        max:       test_point(100.0, 100.0),
        direction: test_point(0.0, 1.0),
        sides:     6,
    };

    let serialized = postcard::to_allocvec(&polygon).expect("Failed to serialize polygon");
    let deserialized: CanvasPolygon =
        postcard::from_bytes(&serialized).expect("Failed to deserialize polygon");

    assert_eq!(polygon, deserialized);
}

#[test]
fn path_empty_round_trip() {
    let path = CanvasPath {
        start_point: test_point(0.0, 0.0),
        actions:     vec![],
    };

    let serialized = postcard::to_allocvec(&path).expect("Failed to serialize path");
    let deserialized: CanvasPath =
        postcard::from_bytes(&serialized).expect("Failed to deserialize path");

    assert_eq!(path, deserialized);
}

#[test]
fn path_with_lines_round_trip() {
    let path = CanvasPath {
        start_point: test_point(10.0, 10.0),
        actions:     vec![
            CanvasPathV1Action::Line(test_point(50.0, 10.0)),
            CanvasPathV1Action::Line(test_point(50.0, 50.0)),
            CanvasPathV1Action::Line(test_point(10.0, 50.0)),
            CanvasPathV1Action::Close,
        ],
    };

    let serialized = postcard::to_allocvec(&path).expect("Failed to serialize path");
    let deserialized: CanvasPath =
        postcard::from_bytes(&serialized).expect("Failed to deserialize path");

    assert_eq!(path, deserialized);
}

#[test]
fn path_with_curves_round_trip() {
    let path = CanvasPath {
        start_point: test_point(0.0, 0.0),
        actions:     vec![
            CanvasPathV1Action::QuadraticCurve {
                end: test_point(100.0, 0.0),
                cp:  test_point(50.0, 50.0),
            },
            CanvasPathV1Action::CubicCurve {
                end: test_point(200.0, 0.0),
                cp1: test_point(133.0, -50.0),
                cp2: test_point(166.0, 50.0),
            },
            CanvasPathV1Action::Move(test_point(300.0, 300.0)),
            CanvasPathV1Action::Line(test_point(400.0, 400.0)),
        ],
    };

    let serialized = postcard::to_allocvec(&path).expect("Failed to serialize path");
    let deserialized: CanvasPath =
        postcard::from_bytes(&serialized).expect("Failed to deserialize path");

    assert_eq!(path, deserialized);
}

// ============================================
// Stable format tests (detect format drift)
// ============================================

// To regenerate these bytes if the format intentionally changes, run:
//   cargo test generate_stable_format_bytes -- --nocapture
#[test]
#[ignore]
fn generate_stable_format_bytes() {
    // Ellipse
    let ellipse = CanvasEllipse {
        min:       test_point(10.0, 20.0),
        max:       test_point(100.0, 200.0),
        direction: test_point(1.0, 0.0),
    };
    let ellipse_bytes = postcard::to_allocvec(&ellipse).unwrap();
    println!("ELLIPSE_BYTES: {:?}", ellipse_bytes);

    // Rectangle
    let rectangle = CanvasRectangle {
        min: test_point(5.0, 15.0),
        max: test_point(50.0, 150.0),
    };
    let rectangle_bytes = postcard::to_allocvec(&rectangle).unwrap();
    println!("RECTANGLE_BYTES: {:?}", rectangle_bytes);

    // Polygon
    let polygon = CanvasPolygon {
        min:       test_point(0.0, 0.0),
        max:       test_point(100.0, 100.0),
        direction: test_point(0.0, 1.0),
        sides:     6,
    };
    let polygon_bytes = postcard::to_allocvec(&polygon).unwrap();
    println!("POLYGON_BYTES: {:?}", polygon_bytes);

    // Path (simple with lines)
    let path = CanvasPath {
        start_point: test_point(10.0, 10.0),
        actions:     vec![
            CanvasPathV1Action::Line(test_point(50.0, 10.0)),
            CanvasPathV1Action::Line(test_point(50.0, 50.0)),
            CanvasPathV1Action::Close,
        ],
    };
    let path_bytes = postcard::to_allocvec(&path).unwrap();
    println!("PATH_SIMPLE_BYTES: {:?}", path_bytes);

    // Path with curves
    let path_curves = CanvasPath {
        start_point: test_point(0.0, 0.0),
        actions:     vec![
            CanvasPathV1Action::QuadraticCurve {
                end: test_point(100.0, 0.0),
                cp:  test_point(50.0, 50.0),
            },
            CanvasPathV1Action::CubicCurve {
                end: test_point(200.0, 0.0),
                cp1: test_point(133.0, -50.0),
                cp2: test_point(166.0, 50.0),
            },
        ],
    };
    let path_curves_bytes = postcard::to_allocvec(&path_curves).unwrap();
    println!("PATH_CURVES_BYTES: {:?}", path_curves_bytes);
}

#[test]
fn ellipse_stable_format() {
    // Expected serialization of:
    // CanvasEllipse { min: (10.0, 20.0), max: (100.0, 200.0), direction: (1.0, 0.0) }
    const ELLIPSE_BYTES: &[u8] = &[
        0, 0, 32, 65, 0, 0, 160, 65, 0, 0, 200, 66, 0, 0, 72, 67, 0, 0, 128, 63, 0, 0, 0, 0,
    ];

    let deserialized: CanvasEllipse =
        postcard::from_bytes(ELLIPSE_BYTES).expect("Failed to deserialize ellipse from stable bytes");

    let expected = CanvasEllipse {
        min:       test_point(10.0, 20.0),
        max:       test_point(100.0, 200.0),
        direction: test_point(1.0, 0.0),
    };

    assert_eq!(deserialized, expected);

    // Also verify that serializing produces the same bytes
    let reserialized = postcard::to_allocvec(&expected).unwrap();
    assert_eq!(reserialized.as_slice(), ELLIPSE_BYTES);
}

#[test]
fn rectangle_stable_format() {
    // Expected serialization of:
    // CanvasRectangle { min: (5.0, 15.0), max: (50.0, 150.0) }
    const RECTANGLE_BYTES: &[u8] = &[
        0, 0, 160, 64, 0, 0, 112, 65, 0, 0, 72, 66, 0, 0, 22, 67,
    ];

    let deserialized: CanvasRectangle =
        postcard::from_bytes(RECTANGLE_BYTES).expect("Failed to deserialize rectangle from stable bytes");

    let expected = CanvasRectangle {
        min: test_point(5.0, 15.0),
        max: test_point(50.0, 150.0),
    };

    assert_eq!(deserialized, expected);

    // Also verify that serializing produces the same bytes
    let reserialized = postcard::to_allocvec(&expected).unwrap();
    assert_eq!(reserialized.as_slice(), RECTANGLE_BYTES);
}

#[test]
fn polygon_stable_format() {
    // Expected serialization of:
    // CanvasPolygon { min: (0.0, 0.0), max: (100.0, 100.0), direction: (0.0, 1.0), sides: 6 }
    const POLYGON_BYTES: &[u8] = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 200, 66, 0, 0, 200, 66, 0, 0, 0, 0, 0, 0, 128, 63, 6,
    ];

    let deserialized: CanvasPolygon =
        postcard::from_bytes(POLYGON_BYTES).expect("Failed to deserialize polygon from stable bytes");

    let expected = CanvasPolygon {
        min:       test_point(0.0, 0.0),
        max:       test_point(100.0, 100.0),
        direction: test_point(0.0, 1.0),
        sides:     6,
    };

    assert_eq!(deserialized, expected);

    // Also verify that serializing produces the same bytes
    let reserialized = postcard::to_allocvec(&expected).unwrap();
    assert_eq!(reserialized.as_slice(), POLYGON_BYTES);
}

#[test]
fn path_simple_stable_format() {
    // Expected serialization of:
    // CanvasPath { start_point: (10.0, 10.0), actions: [Line(50.0, 10.0), Line(50.0, 50.0), Close] }
    const PATH_SIMPLE_BYTES: &[u8] = &[
        0, 0, 32, 65, 0, 0, 32, 65, 3, 2, 0, 0, 72, 66, 0, 0, 32, 65, 2, 0, 0, 72, 66, 0, 0, 72,
        66, 0,
    ];

    let deserialized: CanvasPath =
        postcard::from_bytes(PATH_SIMPLE_BYTES).expect("Failed to deserialize path from stable bytes");

    let expected = CanvasPath {
        start_point: test_point(10.0, 10.0),
        actions:     vec![
            CanvasPathV1Action::Line(test_point(50.0, 10.0)),
            CanvasPathV1Action::Line(test_point(50.0, 50.0)),
            CanvasPathV1Action::Close,
        ],
    };

    assert_eq!(deserialized, expected);

    // Also verify that serializing produces the same bytes
    let reserialized = postcard::to_allocvec(&expected).unwrap();
    assert_eq!(reserialized.as_slice(), PATH_SIMPLE_BYTES);
}

#[test]
fn path_curves_stable_format() {
    // Expected serialization of:
    // CanvasPath { start_point: (0.0, 0.0), actions: [
    //   QuadraticCurve { end: (100.0, 0.0), cp: (50.0, 50.0) },
    //   CubicCurve { end: (200.0, 0.0), cp1: (133.0, -50.0), cp2: (166.0, 50.0) }
    // ] }
    const PATH_CURVES_BYTES: &[u8] = &[
        0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 0, 200, 66, 0, 0, 0, 0, 0, 0, 72, 66, 0, 0, 72, 66, 4,
        0, 0, 72, 67, 0, 0, 0, 0, 0, 0, 5, 67, 0, 0, 72, 194, 0, 0, 38, 67, 0, 0, 72, 66,
    ];

    let deserialized: CanvasPath =
        postcard::from_bytes(PATH_CURVES_BYTES).expect("Failed to deserialize path from stable bytes");

    let expected = CanvasPath {
        start_point: test_point(0.0, 0.0),
        actions:     vec![
            CanvasPathV1Action::QuadraticCurve {
                end: test_point(100.0, 0.0),
                cp:  test_point(50.0, 50.0),
            },
            CanvasPathV1Action::CubicCurve {
                end: test_point(200.0, 0.0),
                cp1: test_point(133.0, -50.0),
                cp2: test_point(166.0, 50.0),
            },
        ],
    };

    assert_eq!(deserialized, expected);

    // Also verify that serializing produces the same bytes
    let reserialized = postcard::to_allocvec(&expected).unwrap();
    assert_eq!(reserialized.as_slice(), PATH_CURVES_BYTES);
}

// ============================================
// Additional edge case tests
// ============================================

#[test]
fn path_all_action_types_round_trip() {
    let path = CanvasPath {
        start_point: test_point(0.0, 0.0),
        actions:     vec![
            CanvasPathV1Action::Move(test_point(10.0, 10.0)),
            CanvasPathV1Action::Line(test_point(20.0, 10.0)),
            CanvasPathV1Action::QuadraticCurve {
                end: test_point(30.0, 10.0),
                cp:  test_point(25.0, 0.0),
            },
            CanvasPathV1Action::CubicCurve {
                end: test_point(40.0, 10.0),
                cp1: test_point(33.0, 20.0),
                cp2: test_point(37.0, 20.0),
            },
            CanvasPathV1Action::Close,
        ],
    };

    let serialized = postcard::to_allocvec(&path).expect("Failed to serialize path");
    let deserialized: CanvasPath =
        postcard::from_bytes(&serialized).expect("Failed to deserialize path");

    assert_eq!(path, deserialized);
}

#[test]
fn shapes_with_negative_coordinates_round_trip() {
    let ellipse = CanvasEllipse {
        min:       test_point(-100.0, -200.0),
        max:       test_point(100.0, 200.0),
        direction: test_point(-1.0, -1.0),
    };

    let serialized = postcard::to_allocvec(&ellipse).expect("Failed to serialize");
    let deserialized: CanvasEllipse =
        postcard::from_bytes(&serialized).expect("Failed to deserialize");

    assert_eq!(ellipse, deserialized);
}

#[test]
fn shapes_with_special_float_values_round_trip() {
    let ellipse = CanvasEllipse {
        min:       test_point(0.0, -0.0),
        max:       test_point(f32::MIN_POSITIVE, f32::MAX),
        direction: test_point(f32::EPSILON, 1e-10),
    };

    let serialized = postcard::to_allocvec(&ellipse).expect("Failed to serialize");
    let deserialized: CanvasEllipse =
        postcard::from_bytes(&serialized).expect("Failed to deserialize");

    assert_eq!(ellipse, deserialized);
}

#[test]
fn polygon_with_many_sides_round_trip() {
    let polygon = CanvasPolygon {
        min:       test_point(0.0, 0.0),
        max:       test_point(100.0, 100.0),
        direction: test_point(1.0, 0.0),
        sides:     1000,
    };

    let serialized = postcard::to_allocvec(&polygon).expect("Failed to serialize");
    let deserialized: CanvasPolygon =
        postcard::from_bytes(&serialized).expect("Failed to deserialize");

    assert_eq!(polygon, deserialized);
}

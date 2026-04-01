///
/// Shapes that can be represented by the shape element
///
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Shape {
    /// A circle
    Circle { center: (f64, f64), point: (f64, f64) },

    /// A rectangle (always axis-aligned)
    Rectangle { center: (f64, f64), point: (f64, f64) },

    /// A polygon
    Polygon { sides: usize, center: (f64, f64), point: (f64, f64) },

}

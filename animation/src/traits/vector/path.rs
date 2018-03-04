///
/// A point in a path
/// 
#[derive(Clone, Copy)]
pub struct PathPoint {
    /// X, Y coordinates of this point
    pub position: (f32, f32)
}

///
/// Represents an element of a bezier path
/// 
#[derive(Clone, Copy)]
pub enum PathElement {
    Move(PathPoint),
    Line(PathPoint),
    Bezier(PathPoint, PathPoint, PathPoint)
}

///
/// Represents a vector path
/// 
#[derive(Clone)]
pub struct Path {
    pub elements: Vec<PathElement>
}

impl Path {
    pub fn new() -> Path {
        Path { elements: vec![] }
    }
}

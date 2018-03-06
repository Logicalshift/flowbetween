///
/// A point in a path
/// 
#[derive(Clone, Copy)]
pub struct PathPoint {
    /// X, Y coordinates of this point
    pub position: (f32, f32)
}

impl PathPoint {
    ///
    /// Creates a new path point
    /// 
    pub fn new(x: f32, y: f32) -> PathPoint {
        PathPoint {
            position: (x, y)
        }
    }

    pub fn x(&self) -> f32 {
        self.position.0
    }

    pub fn y(&self) -> f32 {
        self.position.1
    }
}

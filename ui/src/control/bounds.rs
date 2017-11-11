use super::position::*;

///
/// Represents the bounds of a particular control
///
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub x1: Position,
    pub y1: Position,
    pub x2: Position,
    pub y2: Position
}

impl Bounds {
    ///
    /// Creates a bounding box that fills a container
    ///
    pub fn fill_all() -> Bounds {
        use Position::*;
        Bounds { x1: Start, y1: Start, x2: End, y2: End }
    }

    ///
    /// Bounding box that fills the container vertically and follows the previous control horizontally
    ///
    pub fn next_horiz(width: f32) -> Bounds {
        use Position::*;
        Bounds { x1: After, y1: Start, x2: Offset(width), y2: End }
    }

    ///
    /// Bounding box that fills the container horizontally and follows the previous control horizontally
    ///
    pub fn next_vert(height: f32) -> Bounds {
        use Position::*;
        Bounds { x1: Start, y1: After, x2: End, y2: Offset(height) }
    }

    ///
    /// Bounding box that fills the remaining horizontal space
    ///
    pub fn fill_horiz() -> Bounds {
        use Position::*;
        Bounds { x1: After, y1: Start, x2: End, y2: End }
    }

    ///
    /// Bounding box that fills the remaining vertical space
    ///
    pub fn fill_vert() -> Bounds {
        use Position::*;
        Bounds { x1: Start, y1: After, x2: End, y2: End }
    }
}

///
/// Represents an element in the user interface
///
pub enum Control {
    // Adds an ID to the contained control
    Id(String, Box<Control>),

    /// Control container
    Container(Bounds, Vec<Control>),

    /// Button containing a control
    Button(Bounds, Box<Control>),

    /// Label with some particular text
    Label(Bounds, String)
}

///
/// Represents a position coordinate
///
pub enum Position {
    /// Point located at a specific value
    At(f32),

    /// Point at an offset from its counterpart (eg, width or height)
    Offset(f32),

    /// Point located at the start of the container (ie, left or top depending on if this is an x or y position)
    Start,

    /// Control located at the end of its container (ie, right or bottom depending on if this is an x or y position)
    End,

    /// Same as the last point in this axis (which is 0 initially)
    After
}

///
/// Represents the bounds of a particular control
///
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

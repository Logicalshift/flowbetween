use canvas::*;

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

///
/// Represents an element of a bezier path
/// 
#[derive(Clone, Copy)]
pub enum PathElement {
    Move(PathPoint),
    Line(PathPoint),
    Bezier(PathPoint, PathPoint, PathPoint),
    Close
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

impl From<(f32, f32)> for PathPoint {
    fn from((x, y): (f32, f32)) -> PathPoint {
        PathPoint::new(x, y)
    }
}

impl From<(f64, f64)> for PathPoint {
    fn from((x, y): (f64, f64)) -> PathPoint {
        PathPoint::new(x as f32, y as f32)
    }
}

///
/// Converts a drawing into a path (ignoring all the parts of a drawing
/// that cannot be simply converted)
/// 
impl<Drawing: IntoIterator<Item=Draw>> From<Drawing> for Path {
    fn from(drawing: Drawing) -> Path {
        let elements = drawing.into_iter()
            .map(|draw| {
                use self::Draw::*;

                match draw {
                    Move(x, y)              => Some(PathElement::Move(PathPoint::from((x, y)))),
                    Line(x, y)              => Some(PathElement::Line(PathPoint::from((x, y)))),
                    BezierCurve(p, c1, c2)  => Some(PathElement::Bezier(PathPoint::from(p), PathPoint::from(c1), PathPoint::from(c2))),
                    ClosePath               => Some(PathElement::Close),

                    _                       => None
                }
            })
            .filter(|element| element.is_some())
            .map(|element| element.unwrap());
        
        Path {
            elements: elements.collect()
        }
    }
}

///
/// Converts a path into a drawing
/// 
impl<'a> Into<Vec<Draw>> for &'a Path {
    fn into(self) -> Vec<Draw> {
        let drawing = self.elements.iter()
            .map(|path| {
                use self::Draw::*;

                match path {
                    &PathElement::Move(point)       => Move(point.x(), point.y()),
                    &PathElement::Line(point)       => Line(point.x(), point.y()),
                    &PathElement::Bezier(p, c1, c2) => BezierCurve(p.position, c1.position, c2.position),
                    &PathElement::Close             => ClosePath
                }
            });
        
        drawing.collect()
    }
}

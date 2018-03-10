use super::point::*;
use super::element::*;

use canvas::*;

///
/// Represents a vector path
/// 
#[derive(Clone)]
pub struct Path {
    pub elements: Vec<PathElement>
}

impl Path {
    ///
    /// Creates a new, empty path
    /// 
    pub fn new() -> Path {
        Path { elements: vec![] }
    }

    ///
    /// Creates a path from an elements iterator
    /// 
    pub fn from_elements<Elements: IntoIterator<Item=PathElement>>(elements: Elements) -> Path {
        Path {
            elements: elements.into_iter().collect()
        }
    }

    ///
    /// Creates a path from a drawing iterator
    /// 
    pub fn from_drawing<Drawing: IntoIterator<Item=Draw>>(drawing: Drawing) -> Path {
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

impl From<Vec<PathElement>> for Path {
    fn from(points: Vec<PathElement>) -> Path {
        Path {
            elements: points
        }
    }
}

///
/// Converts a drawing into a path (ignoring all the parts of a drawing
/// that cannot be simply converted)
/// 
impl From<Vec<Draw>> for Path {
    fn from(drawing: Vec<Draw>) -> Path {
        Path::from_drawing(drawing)
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

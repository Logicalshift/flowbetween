use super::point::*;
use super::element::*;

use canvas::*;
use curves::geo::*;
use curves::bezier::path::*;

use std::iter;

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

impl Geo for Path {
    type Point = PathPoint;
}

impl BezierPath for Path {
    /// Type of an iterator over the points in this curve. This tuple contains the points ordered as a hull: ie, two control points followed by a point on the curve
    type PointIter = Box<dyn Iterator<Item=(Self::Point, Self::Point, Self::Point)>>;

    ///
    /// Retrieves the initial point of this path
    /// 
    fn start_point(&self) -> Self::Point {
        use self::PathElement::*;

        match self.elements[0] {
            Move(p)         => p,
            Line(p)         => p,
            Bezier(p, _, _) => p,
            Close           => PathPoint::new(0.0, 0.0)
        }
    }

    ///
    /// Retrieves an iterator over the points in this path
    /// 
    /// Note that the bezier path trait doesn't support Move or Close operations so these will be ignored.
    /// Operations like `path_contains_point` may be unreliable for 'broken' paths as a result of this limitation.
    /// (Paths with a 'move' in them are really two seperate paths)
    /// 
    fn points(&self) -> Self::PointIter {
        // Points as a set of bezier curves
        let mut points = vec![];

        // The last point, which is used to generate the set of points along a line
        let start_point     = self.start_point();
        let mut last_point  = start_point;

        // Convert each element to a point in the path
        for element in self.elements.iter() {
            match element {
                PathElement::Bezier(target, cp1, cp2) => {
                    points.push((*cp1, *cp2, *target));
                    last_point = *target;
                },

                PathElement::Line(target)  |
                PathElement::Move(target) => {
                    // Generate control points for a line
                    let diff = *target-last_point;
                    let cp1 = diff * 0.3;
                    let cp2 = diff * 0.7;
                    let cp1 = last_point + cp1;
                    let cp2 = last_point + cp2;

                    points.push((cp1, cp2, *target));
                    last_point = *target;
                },

                PathElement::Close => { 
                    // Line to the start point
                    let diff = start_point-last_point;
                    let cp1 = diff * 0.3;
                    let cp2 = diff * 0.7;
                    let cp1 = last_point + cp1;
                    let cp2 = last_point + cp2;

                    points.push((cp1, cp2, start_point));
                    last_point = start_point;
                }
            }
        }

        // Turn into an iterator
        Box::new(points.into_iter())
    }
}

impl BezierPathFactory for Path {
    ///
    /// Creates a new instance of this path from a set of points
    /// 
    fn from_points<FromIter: IntoIterator<Item=(Self::Point, Self::Point, Self::Point)>>(start_point: Self::Point, points: FromIter) -> Self {
        let elements = points.into_iter()
            .map(|(cp1, cp2, target)| PathElement::Bezier(target, cp1, cp2));
        let elements = iter::once(PathElement::Move(start_point))
            .chain(elements);

        Path {
            elements: elements.collect()
        }
    }
}
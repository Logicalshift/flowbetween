use super::point::*;
use super::curve::*;
use super::component::*;

use flo_canvas::*;
use flo_curves::geo::*;
use flo_curves::line::*;
use flo_curves::bezier::*;
use flo_curves::bezier::path::*;

use itertools::*;

use std::iter;
use std::sync::*;

///
/// Represents a vector path
/// 
#[derive(Clone, Debug)]
pub struct Path {
    pub elements: Arc<Vec<PathComponent>>
}

impl Path {
    ///
    /// Creates a new, empty path
    /// 
    pub fn new() -> Path {
        Path { elements: Arc::new(vec![]) }
    }

    ///
    /// Creates a path from an elements iterator
    /// 
    pub fn from_elements<Elements: IntoIterator<Item=PathComponent>>(elements: Elements) -> Path {
        Path {
            elements: Arc::new(elements.into_iter().collect())
        }
    }

    ///
    /// The number of elements in this path
    ///
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    ///
    /// Returns the elements that make up this path as an iterator
    ///
    pub fn elements<'a>(&'a self) -> impl 'a+Iterator<Item=PathComponent> {
        self.elements.iter()
            .map(|component| *component)
    }

    ///
    /// Returns references to the elements that make up this path as an iterator
    ///
    pub fn elements_ref<'a>(&'a self) -> impl 'a+Iterator<Item=&'a PathComponent> {
        self.elements.iter()
    }

    ///
    /// Creates a path from an existing collection of components without copying them
    ///
    pub fn from_elements_arc(elements: Arc<Vec<PathComponent>>) -> Path {
        Path {
            elements
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
                    Move(x, y)              => Some(PathComponent::Move(PathPoint::from((x, y)))),
                    Line(x, y)              => Some(PathComponent::Line(PathPoint::from((x, y)))),
                    BezierCurve(p, c1, c2)  => Some(PathComponent::Bezier(PathPoint::from(p), PathPoint::from(c1), PathPoint::from(c2))),
                    ClosePath               => Some(PathComponent::Close),

                    _                       => None
                }
            })
            .filter(|element| element.is_some())
            .map(|element| element.unwrap());
        
        Path {
            elements: Arc::new(elements.collect())
        }
    }

    ///
    /// Returns this path as an iterator of draw elements
    ///
    pub fn to_drawing<'a>(&'a self) -> impl 'a+Iterator<Item=Draw> {
        self.elements.iter()
            .map(|path| {
                use self::Draw::*;

                match path {
                    &PathComponent::Move(point)       => Move(point.x(), point.y()),
                    &PathComponent::Line(point)       => Line(point.x(), point.y()),
                    &PathComponent::Bezier(p, c1, c2) => BezierCurve(p.position, c1.position, c2.position),
                    &PathComponent::Close             => ClosePath
                }
            })
    }

    ///
    /// Returns the curves in this path
    ///
    pub fn to_curves<'a>(&'a self) -> impl 'a+Iterator<Item=PathCurve> {
        self.elements.iter()
            .tuple_windows()
            .flat_map(|(prev, next)| {
                use self::PathComponent::*;

                let start_point = match prev {
                    Move(p)          => p,
                    Line(p)          => p,
                    Bezier(p, _, _)  => p,
                    Close            => { return None }
                };

                match next {
                    Move(_)                     => None,
                    Close                       => None,
                    Line(end_point)             => Some(line_to_bezier(&(*start_point, *end_point))),
                    Bezier(end_point, cp1, cp2) => Some(PathCurve::from_points(*start_point, (*cp1, *cp2), *end_point))
                }
            })
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

impl From<Vec<PathComponent>> for Path {
    fn from(points: Vec<PathComponent>) -> Path {
        Path {
            elements: Arc::new(points)
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
        self.to_drawing().collect()
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
        use self::PathComponent::*;

        if self.elements.len() > 0 {
            match self.elements[0] {
                Move(p)         => p,
                Line(p)         => p,
                Bezier(p, _, _) => p,
                Close           => PathPoint::new(0.0, 0.0)
            }
        } else {
            PathPoint::new(0.0, 0.0)
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
                PathComponent::Bezier(target, cp1, cp2) => {
                    points.push((*cp1, *cp2, *target));
                    last_point = *target;
                },

                PathComponent::Line(target)  |
                PathComponent::Move(target) => {
                    // Generate control points for a line
                    let diff = *target-last_point;
                    let cp1 = diff * 0.3;
                    let cp2 = diff * 0.7;
                    let cp1 = last_point + cp1;
                    let cp2 = last_point + cp2;

                    points.push((cp1, cp2, *target));
                    last_point = *target;
                },

                PathComponent::Close => { 
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
            .map(|(cp1, cp2, target)| PathComponent::Bezier(target, cp1, cp2));
        let elements = iter::once(PathComponent::Move(start_point))
            .chain(elements);

        Path {
            elements: Arc::new(elements.collect())
        }
    }
}
use super::path::*;
use super::rect::*;
use super::point::*;
use super::curve::*;
use super::element::*;

use curves::*;

///
/// Trait implemented by graphical elements with a bounding box
/// 
pub trait HasBoundingBox {
    ///
    /// Retrieves the bounding box for this element
    /// 
    fn bounding_box(&self) -> Rect;
}

///
/// An element and its starting point can be converted into a bounding rectangle
/// 
impl From<(PathPoint, PathElement)> for Rect {
    fn from((start_point, element): (PathPoint, PathElement)) -> Rect {
        use self::PathElement::*;

        match element {
            Move(point)     => Rect::new(start_point, point).normalize(),
            Line(point)     => Rect::new(start_point, point).normalize(),
            Close           => Rect::new(start_point, start_point).normalize(),

            Bezier(point, cp1, cp2) => {
                let curve                   = PathCurve(start_point, Bezier(point, cp1, cp2));
                let (topleft, bottomright)  = curve.bounding_box();

                Rect::new(topleft, bottomright).normalize()
            }
        }
    }
}

///
/// Pairs of path elements can also be turned into bounding rectangles
/// 
impl From<(PathElement, PathElement)> for Rect {
    fn from((previous, next): (PathElement, PathElement)) -> Rect {
        use self::PathElement::*;

        if let Close = previous {
            // If the previous element is a 'close' element then we only include one point
            let only_point = match next {
                Move(point)                 => point,
                Line(point)                 => point,
                Bezier(point, _cp1, _cp2)   => point,
                Close                       => PathPoint::origin()
            };

            Rect::new(only_point, only_point)
        } else {
            // The last point of the previous element is the start point
            let start_point = match previous {
                Move(point)                 => point,
                Line(point)                 => point,
                Bezier(point, _cp1, _cp2)   => point,
                Close                       => PathPoint::origin()
            };

            // Use this to generate the bounding rectangle
            Rect::from((start_point, next))
        }
    }
}

///
/// Paths can be converted into bounding boxes
/// 
impl<'a> From<&'a Path> for Rect {
    fn from(p: &'a Path) -> Rect {
        // We want all the pairs of elements
        let previous    = p.elements.iter().take(p.elements.len()-1);
        let following   = p.elements.iter().skip(1);
        let mut bounds  = Rect::empty();

        // The bounds of the path is the union of the bounds of all of the elements
        for (previous, next) in previous.zip(following) {
            bounds = bounds.union(Rect::from((*previous, *next)));
        }

        bounds
    }
}

impl From<Path> for Rect {
    #[inline]
    fn from(p: Path) -> Rect {
        Rect::from(&p)
    }
}

impl HasBoundingBox for Path {
    #[inline]
    fn bounding_box(&self) -> Rect {
        Rect::from(self)
    }
}
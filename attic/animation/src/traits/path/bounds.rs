use super::path::*;
use super::rect::*;
use super::point::*;
use super::curve::*;
use super::component::*;

use flo_curves::*;

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
impl From<(PathPoint, PathComponent)> for Rect {
    fn from((start_point, element): (PathPoint, PathComponent)) -> Rect {
        use self::PathComponent::*;

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
impl From<(PathComponent, PathComponent)> for Rect {
    fn from((previous, next): (PathComponent, PathComponent)) -> Rect {
        use self::PathComponent::*;

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
        if p.elements.len() == 0 {
            Rect::empty()
        } else {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_bounding_box_for_line_element() {
        let bounds = Rect::from((PathPoint::new(30.0, 30.0), PathComponent::Line(PathPoint::new(60.0, 20.0))));

        assert!(bounds.x1 == 30.0);
        assert!(bounds.y1 == 20.0);
        assert!(bounds.x2 == 60.0);
        assert!(bounds.y2 == 30.0);
    }

    #[test]
    fn can_get_bounding_box_for_move_and_line_element() {
        let bounds = Rect::from((PathComponent::Move(PathPoint::new(30.0, 30.0)), PathComponent::Line(PathPoint::new(60.0, 20.0))));

        assert!(bounds.x1 == 30.0);
        assert!(bounds.y1 == 20.0);
        assert!(bounds.x2 == 60.0);
        assert!(bounds.y2 == 30.0);
    }

    #[test]
    fn can_get_bounding_box_for_line_path() {
        use self::PathComponent::*;

        let line_path = Path::from_elements(vec![
            Move(PathPoint::new(30.0, 30.0)),
            Line(PathPoint::new(60.0, 20.0))
        ]);

        let bounds = line_path.bounding_box();

        assert!(bounds.x1 == 30.0);
        assert!(bounds.y1 == 20.0);
        assert!(bounds.x2 == 60.0);
        assert!(bounds.y2 == 30.0);
    }

    #[test]
    fn can_get_bounding_box_for_triangle_path() {
        use self::PathComponent::*;

        let line_path = Path::from_elements(vec![
            Move(PathPoint::new(30.0, 30.0)),
            Line(PathPoint::new(60.0, 20.0)),
            Line(PathPoint::new(120.0, 50.0))
        ]);

        let bounds = line_path.bounding_box();

        assert!(bounds.x1 == 30.0);
        assert!(bounds.y1 == 20.0);
        assert!(bounds.x2 == 120.0);
        assert!(bounds.y2 == 50.0);
    }

    #[test]
    fn can_get_bounding_box_for_simple_line_bezier_path() {
        use self::PathComponent::*;

        let line_path = Path::from_elements(vec![
            Move(PathPoint::new(30.0, 30.0)),
            Bezier(PathPoint::new(60.0, 60.0), PathPoint::new(40.0, 40.0), PathPoint::new(50.0, 50.0))
        ]);

        let bounds = line_path.bounding_box();

        assert!(bounds.x1 == 30.0);
        assert!(bounds.y1 == 30.0);
        assert!(bounds.x2 == 60.0);
        assert!(bounds.y2 == 60.0);
    }

    #[test]
    fn can_get_bounding_box_for_curved_bezier_path() {
        use self::PathComponent::*;

        let line_path = Path::from_elements(vec![
            Move(PathPoint::new(0.0, 1.0)),
            Bezier(PathPoint::new(2.0, 3.0), PathPoint::new(-1.1875291, 1.5), PathPoint::new(1.5, 2.5))
        ]);

        let bounds = line_path.bounding_box();

        assert!((bounds.x1- -0.3).abs() < 0.0001);
        assert!((bounds.y1- 1.0).abs() < 0.0001);
        assert!((bounds.x2- 2.0).abs() < 0.0001);
        assert!((bounds.y2- 3.0).abs() < 0.0001);
    }
}

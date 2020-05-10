use super::*;

use flo_canvas::*;
use flo_curves::geo::*;

use std::f32;

///
/// Represents a rectangle
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Rect {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32
}

impl Rect {
    ///
    /// Creates a new rectangle
    ///
    pub fn new(top_left: PathPoint, bottom_right: PathPoint) -> Rect {
        Rect {
            x1: top_left.position.0 as f32,
            y1: top_left.position.1 as f32,
            x2: bottom_right.position.0 as f32,
            y2: bottom_right.position.1 as f32
        }
    }

    ///
    /// Creates a new rectangle with specific points
    ///
    pub fn with_points(x1: f32, y1: f32, x2: f32, y2: f32) -> Rect {
        Rect {
            x1, y1, x2, y2
        }
    }

    ///
    /// Creates an empty rectangle
    ///
    pub fn empty() -> Rect {
        Rect {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0
        }
    }

    ///
    /// Creates a new rectangle that's inset from this one by a certain amount
    ///
    pub fn inset(&self, x_distance: f32, y_distance: f32) -> Rect {
        let half_x = x_distance/2.0;
        let half_y = y_distance/2.0;

        Rect {
            x1: f32::min(self.x1, self.x2) + half_x,
            y1: f32::min(self.y1, self.y2) + half_y,
            x2: f32::max(self.x1, self.x2) - half_x,
            y2: f32::max(self.y1, self.y2) - half_y,
        }
    }

    ///
    /// Converts a rectangle into one where x2 and y2 are greater than x1 and y1
    ///
    #[inline]
    pub fn normalize(self) -> Rect {
        Rect {
            x1: f32::min(self.x1, self.x2),
            y1: f32::min(self.y1, self.y2),
            x2: f32::max(self.x1, self.x2),
            y2: f32::max(self.y1, self.y2),
        }
    }

    ///
    /// Returns true if the specified point is inside this rectangle
    ///
    pub fn contains(&self, x: f32, y: f32) -> bool {
        (f32::min(self.x1, self.x2) <= x)   &&
        (x <= f32::max(self.x1, self.x2))   &&
        (f32::min(self.y1, self.y2) <= y)   &&
        (y <= f32::max(self.y1, self.y2))
    }

    ///
    /// Returns true if the specified rectangle overlaps this one
    ///
    pub fn overlaps(&self, other: &Rect) -> bool {
        f32::min(self.x1, self.x2) < f32::max(other.x1, other.x2)
        && f32::max(self.x1, self.x2) > f32::min(other.x1, other.x2)
        && f32::min(self.y1, self.y2) < f32::max(other.y1, other.y2)
        && f32::max(self.y1, self.y2) > f32::min(other.y1, other.y2)
    }

    ///
    /// Returns the center of this rectangle
    ///
    pub fn center(&self) -> Coord2 {
        Coord2(((self.x1+self.x2)/2.0) as f64, ((self.y1+self.y2)/2.0) as f64)
    }

    ///
    /// True if this rectangle has no size
    ///
    #[inline]
    pub fn is_zero_size(&self) -> bool {
        self.x1 == self.x2 && self.y1 == self.y2
    }

    ///
    /// Creates the union of two rectangles
    ///
    #[inline]
    pub fn union(self, rhs: Rect) -> Rect {
        if self.is_zero_size() {
            rhs
        } else if rhs.is_zero_size() {
            self
        } else {
            Rect {
                x1: f32::min(self.x1, f32::min(self.x2, f32::min(rhs.x1, rhs.x2))),
                y1: f32::min(self.y1, f32::min(self.y2, f32::min(rhs.y1, rhs.y2))),
                x2: f32::max(self.x1, f32::max(self.x2, f32::max(rhs.x1, rhs.x2))),
                y2: f32::max(self.y1, f32::max(self.y2, f32::max(rhs.y1, rhs.y2))),
            }
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.x2 - self.x1
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.y2 - self.y1
    }

    ///
    /// Draws this rectangle on a graphics context
    ///
    pub fn draw(&self, gc: &mut dyn GraphicsContext) {
        gc.move_to(self.x1, self.y1);
        gc.line_to(self.x2, self.y1);
        gc.line_to(self.x2, self.y2);
        gc.line_to(self.x1, self.y2);
        gc.close_path();
    }
}

impl Geo for Rect {
    type Point = PathPoint;
}

impl BoundingBox for Rect {
    fn from_min_max(min: Self::Point, max: Self::Point) -> Self {
        Rect::new(min, max)
    }

    fn min(&self) -> Self::Point {
        PathPoint::new(self.x1, self.y1)
    }

    fn max(&self) -> Self::Point {
        PathPoint::new(self.x2, self.y2)
    }

    fn empty() -> Self {
        Rect {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_zero_size()
    }

    fn union_bounds(self, target: Self) -> Self {
        self.union(target)
    }
}

impl<'a> Into<Vec<Draw>> for &'a Rect {
    fn into(self) -> Vec<Draw> {
        vec![
            Draw::Move(self.x1, self.y1),
            Draw::Line(self.x2, self.y1),
            Draw::Line(self.x2, self.y2),
            Draw::Line(self.x1, self.y2),
            Draw::ClosePath
        ]
    }
}

impl Into<Vec<Draw>> for Rect {
    fn into(self) -> Vec<Draw> {
        (&self).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_union_rects() {
        assert!(Rect::with_points(30.0, 30.0, 60.0, 40.0).union(Rect::with_points(40.0, 20.0, 100.0, 30.0)) == Rect::with_points(30.0, 20.0, 100.0, 40.0));
    }

    #[test]
    fn can_union_empty_rects_lhs() {
        assert!(Rect::empty().union(Rect::with_points(40.0, 20.0, 100.0, 30.0)) == Rect::with_points(40.0, 20.0, 100.0, 30.0));
    }

    #[test]
    fn can_union_empty_rects_rhs() {
        assert!(Rect::with_points(30.0, 30.0, 60.0, 40.0).union(Rect::empty()) == Rect::with_points(30.0, 30.0, 60.0, 40.0));
    }

    #[test]
    fn overlapping_rects() {
        let r1 = Rect::with_points(30.0, 30.0, 60.0, 40.0);
        let r2 = Rect::with_points(20.0, 25.0, 35.0, 35.0);

        assert!(r1.overlaps(&r2));
    }

    #[test]
    fn non_overlapping_rects() {
        let r1 = Rect::with_points(30.0, 30.0, 60.0, 40.0);
        let r2 = Rect::with_points(20.0, 25.0, 9.0, 10.0);

        assert!(!r1.overlaps(&r2));
    }

    #[test]
    fn touching_rects() {
        let r1 = Rect::with_points(30.0, 30.0, 60.0, 40.0);
        let r2 = Rect::with_points(20.0, 25.0, 30.0, 30.0);

        assert!(!r1.overlaps(&r2));
    }

    #[test]
    fn overlap_interior_rect() {
        let r1 = Rect::with_points(30.0, 30.0, 60.0, 40.0);
        let r2 = r1.inset(10.0, 10.0);

        assert!(r1.overlaps(&r2));
    }

    #[test]
    fn overlap_exterior_rect() {
        let r1 = Rect::with_points(30.0, 30.0, 60.0, 40.0);
        let r2 = r1.inset(-10.0, -10.0);

        assert!(r1.overlaps(&r2));
    }
}

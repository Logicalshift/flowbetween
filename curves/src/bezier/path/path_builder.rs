use super::path::*;

///
/// Used to build a bezier path
/// 
pub struct BezierPathBuilder<P: BezierPath> {
    /// Where the path starts
    start_point: P::Point,

    /// The points in the path
    points: Vec<(P::Point, P::Point, P::Point)>
}

impl<P: BezierPathFactory> BezierPathBuilder<P> {
    ///
    /// Creates a new bezier path builder with the specified start point
    /// 
    pub fn start(start: P::Point) -> BezierPathBuilder<P> {
        BezierPathBuilder {
            start_point:    start,
            points:         vec![]
        }
    }

    ///
    /// Builds the path for this builder
    /// 
    pub fn build(self) -> P {
        P::from_points(self.start_point, self.points)
    }

    ///
    /// Adds a line to the specified point
    /// 
    pub fn line_to(mut self, point: P::Point) -> Self {
        // Get the vector from the last point to the new point
        let distance = if self.points.len() == 0 {
            point - self.start_point
        } else {
            point - self.points[self.points.len()-1].2
        };

        // A line puts control points at 33% and 66% of the distance
        let cp1 = point - (distance*0.6666);
        let cp2 = point - (distance*0.3333);

        self.points.push((cp1, cp2, point));

        self
    }

    ///
    /// Adds a curve to a particular point
    /// 
    pub fn curve_to(mut self, (cp1, cp2): (P::Point, P::Point), end_point: P::Point) -> Self {
        self.points.push((cp1, cp2, end_point));

        self
    }
}
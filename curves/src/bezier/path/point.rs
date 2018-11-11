use super::ray::*;
use super::path::*;
use super::to_curves::*;
use super::graph_path::*;
use super::super::curve::*;
use super::super::normal::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

///
/// Represents a curve that can be represented either forwards or backwards
///
#[derive(Clone)]
pub (crate) enum ReversableCurve<Curve> {
    Forward(Curve),
    Reversed(Curve)
}

impl<Curve: BezierCurve> Geo for ReversableCurve<Curve> {
    type Point=Curve::Point;
}

impl<Curve: BezierCurve> BezierCurve for ReversableCurve<Curve> {
    #[inline]
    fn start_point(&self) -> Curve::Point { 
        match self {
            ReversableCurve::Forward(curve)     => curve.start_point(),
            ReversableCurve::Reversed(curve)    => curve.end_point()
        }
    }

    #[inline]
    fn end_point(&self) -> Curve::Point { 
        match self {
            ReversableCurve::Forward(curve)     => curve.end_point(),
            ReversableCurve::Reversed(curve)    => curve.start_point()
        }
    }

    #[inline]
    fn control_points(&self) -> (Curve::Point, Curve::Point) {
        match self {
            ReversableCurve::Forward(curve)     => curve.control_points(),
            ReversableCurve::Reversed(curve)    => {
                let (cp1, cp2) = curve.control_points();
                (cp2, cp1)
            }
        }
    }
}

impl<Curve: BezierCurve> RayPath for Vec<Curve> 
where Curve::Point: Coordinate2D {
    type Curve = ReversableCurve<Curve>;
    type Point = Curve::Point;

    #[inline] fn num_points(&self) -> usize {
        self.len()
    }

    #[inline] fn num_edges(&self, _point_idx: usize) -> usize {
        1
    }

    #[inline] fn reverse_edges_for_point(&self, point_idx: usize) -> Vec<GraphEdgeRef> {
        if point_idx == 0 {
            vec![GraphEdgeRef { start_idx: self.len()-1, edge_idx: 0, reverse: true }]
        } else {
            vec![GraphEdgeRef { start_idx: point_idx-1, edge_idx: 0, reverse: true }]
        }
    }

    #[inline] fn edges_for_point(&self, point_idx: usize) -> Vec<GraphEdgeRef> {
        vec![GraphEdgeRef { start_idx: point_idx, edge_idx: 0, reverse: false }]
    }

    #[inline] fn get_edge(&self, edge: GraphEdgeRef) -> Self::Curve {
        if edge.reverse {
            ReversableCurve::Reversed(self[edge.start_idx].clone())
        } else {
            ReversableCurve::Forward(self[edge.start_idx].clone())
        }
    }

    #[inline] fn get_next_edge(&self, edge: GraphEdgeRef) -> (GraphEdgeRef, Self::Curve) {
        let next_ref = GraphEdgeRef { start_idx: self.edge_end_point_idx(edge), edge_idx: 0, reverse: edge.reverse };
        (next_ref, self.get_edge(next_ref))
    }

    #[inline] fn point_position(&self, point: usize) -> Self::Point {
        self[point].start_point()
    }

    #[inline] fn edge_start_point_idx(&self, edge: GraphEdgeRef) -> usize {
        if edge.reverse {
            unimplemented!()
        } else {
            edge.start_idx
        }
    }

    #[inline] fn edge_end_point_idx(&self, edge: GraphEdgeRef) -> usize {
        if edge.reverse {
            unimplemented!()
        } else {
            if edge.start_idx+1 == self.len() {
                0
            } else {
                edge.start_idx+1
            }
        }
    }

    #[inline] fn edge_following_edge_idx(&self, _edge: GraphEdgeRef) -> usize {
        0
    }
}

///
/// Returns true if a particular point is within a bezier path
/// 
pub fn path_contains_point<P: BezierPath>(path: &P, point: &P::Point) -> bool
where P::Point: Coordinate2D {
    // We want to cast a ray from the outer edge of the bounds to our point
    let (min_bounds, max_bounds) = path.bounding_box();

    if min_bounds.x() > point.x() || max_bounds.x() < point.x() || min_bounds.y() > point.y() || max_bounds.y() < point.y() {
        // Point is outside the bounds of the path
        false
    } else {
        // Ray is from the top of the bounds to our point
        let ray             = (max_bounds + P::Point::from_components(&[0.01, 0.01]), *point);
        let ray_direction   = ray.1 - ray.0;

        // Call through to ray_collisions to get the collisions
        let curves          = path_to_curves::<_, Curve<_>>(path).collect::<Vec<_>>();
        let collisions      = ray_collisions(&curves, &ray);

        // The total of all of the ray directions
        let mut total_direction     = 0;

        for (collision, curve_t, line_t, _pos) in collisions {
            // Stop once the ray reaches the desired point
            if line_t > 1.0 { break; }

            // Curve this collision was is just the start index of the edge
            let curve_idx   = collision.edge().start_idx;

            // Use the normal at this point to determine the direction relative to the ray
            let normal      = curves[curve_idx].normal_at_pos(curve_t);
            let direction   = ray_direction.dot(&normal).signum() as i32;

            // Add to the total direction
            total_direction += direction;
        }

        // Point is inside the path if the ray crosses more lines facing in a particular direction
        total_direction != 0
    }
}

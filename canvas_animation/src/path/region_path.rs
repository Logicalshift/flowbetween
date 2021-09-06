use flo_canvas::*;
use flo_curves::*;
use flo_curves::bezier::path::*;

///
/// How a path fits in an animation region
///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PathRegionType {
    /// No part of the path is inside the region
    OutsideRegion,

    /// Entirely encloses the region
    EnclosesRegion,

    /// Partially inside, partially outside the region
    IntersectsRegion,

    /// Entirely inside the region
    InsideRegion,
}

///
/// The kinds of exterior edges currently represented in the GraphPath in the RegionPath
///
#[derive(Copy, Clone, PartialEq, Eq)]
enum RegionExteriorEdgeKind {
    /// The exterior edges represent the intersection of the path and the region
    Intersection,

    /// The exterior edges represent the subtraction of the path and the region
    Subtraction
}

///
/// Represents a path that has been intersected with a region
///
pub struct RegionPath {
    /// The graph path representing the intersection between the main path (path 0) and the region (path 1)
    graph: GraphPath<Coord2, PathLabel>,

    /// How the graph is set up with exterior edges
    exterior_edges: RegionExteriorEdgeKind
}

impl RegionPath {
    ///
    /// Creates a new region path from a graph path, set to the intersection of the animation and its region
    ///
    pub (crate) fn new(path: GraphPath<Coord2, PathLabel>) -> RegionPath {
        RegionPath {
            graph:          path,
            exterior_edges: RegionExteriorEdgeKind::Intersection
        }
    }

    ///
    /// Recalculates the graph so that it represents the intersections between the path and the region
    ///
    fn to_path_intersection(&mut self) {
        self.graph.reset_edge_kinds();
        self.graph.set_exterior_by_intersecting();
        self.graph.heal_exterior_gaps();

        self.exterior_edges = RegionExteriorEdgeKind::Intersection;
    }

    ///
    /// Recalculates the graph so that it represents the part of the path outside of the region
    ///
    fn to_path_subtraction(&mut self) {
        self.graph.reset_edge_kinds();
        self.graph.set_exterior_by_subtracting();
        self.graph.heal_exterior_gaps();

        self.exterior_edges = RegionExteriorEdgeKind::Subtraction;
    }

    ///
    /// Generates the path operations for the current state of the path
    ///
    fn to_path_ops(&self) -> Vec<PathOp> {
        // Convert to SimpleBezierPaths
        let simple_paths = self.graph.exterior_paths::<SimpleBezierPath>();

        // Convert to PathsOps
        let mut result = vec![];
        for (start_point, curve_points) in simple_paths.into_iter() {
            // Move to start the subpath
            result.push(PathOp::Move(start_point.x() as _, start_point.y() as _));

            // Add the other points as bezier curves
            for (cp1, cp2, end_point) in curve_points {
                result.push(PathOp::BezierCurve(((cp1.x() as _, cp1.y() as _), (cp2.x() as _, cp2.y() as _)), (end_point.x() as _, end_point.y() as _)));
            }

            // All paths are closed (TODO: support open paths - FlowBetween currently doesn't generate these)
            result.push(PathOp::ClosePath);
        }

        result
    }

    ///
    /// Returns the type of region matched by this region type
    ///
    pub fn region_type(&mut self) -> PathRegionType {
        if self.exterior_edges != RegionExteriorEdgeKind::Intersection { self.to_path_intersection(); }

        let mut num_path_edges          = 0;
        let mut num_exterior_edges      = 0;
        let mut num_path_exterior_edges = 0;

        for edge_ref in self.graph.all_edge_refs() {
            if self.graph.edge_label(edge_ref).0 == 0 {
                num_path_edges += 1;

                if self.graph.edge_kind(edge_ref) == GraphPathEdgeKind::Exterior {
                    num_exterior_edges      += 1;
                    num_path_exterior_edges += 1;
                }
            } else if self.graph.edge_kind(edge_ref) == GraphPathEdgeKind::Exterior {
                num_exterior_edges += 1;
            }
        }

        if num_path_edges == num_path_exterior_edges && num_path_edges == num_exterior_edges {
            // The intersection is the entire path, so it's inside the region
            PathRegionType::InsideRegion
        } else if num_path_exterior_edges > 0 {
            // At least one edge from the original path is inside the region
            PathRegionType::IntersectsRegion
        } else if num_exterior_edges > 0 {
            // The region overlaps the path but no edges cross it
            PathRegionType::EnclosesRegion
        } else {
            // No exterior edges
            PathRegionType::OutsideRegion
        }
    }

    ///
    /// Returns the definition for the part of the path that's inside the region
    ///
    pub fn path_inside(&mut self) -> Vec<PathOp> {
        if self.exterior_edges != RegionExteriorEdgeKind::Intersection { self.to_path_intersection(); }
        self.to_path_ops()
    }

    ///
    /// Returns the definition for the part of the path that's outside of the region
    ///
    pub fn path_outside(&mut self) -> Vec<PathOp> {
        if self.exterior_edges != RegionExteriorEdgeKind::Subtraction { self.to_path_subtraction(); }
        self.to_path_ops()
    }
}

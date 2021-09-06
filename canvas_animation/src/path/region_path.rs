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
}

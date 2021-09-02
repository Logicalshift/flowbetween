use flo_curves::*;
use flo_curves::bezier::path::*;

///
/// How a path fits in an animation region
///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GraphRegionType {
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
/// Represents a path that has been 
///
pub struct RegionPath {
    /// The graph path representing the intersection between the main path (path 0) and the region (path 1)
    graph: GraphPath<Coord2, PathLabel>
}

impl RegionPath {
    ///
    /// Creates a new region path from a graph path
    ///
    pub (crate) fn new(path: GraphPath<Coord2, PathLabel>) -> RegionPath {
        RegionPath {
            graph: path
        }
    }

    ///
    /// Returns the type of region matched by this region type
    ///
    pub fn region_type(&self) -> GraphRegionType {
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
            GraphRegionType::InsideRegion
        } else if num_path_exterior_edges > 0 {
            // At least one edge from the original path is inside the region
            GraphRegionType::IntersectsRegion
        } else if num_exterior_edges > 0 {
            // The region overlaps the path but no edges cross it
            GraphRegionType::EnclosesRegion
        } else {
            // No exterior edges
            GraphRegionType::OutsideRegion
        }
    }
}

use super::region_path::*;
use super::animation_path::*;
use crate::region::*;

use flo_curves::*;
use flo_curves::bezier::path::*;

use std::time::{Duration};

impl AnimationPath {
    ///
    /// Returns true if this path is overlaps the specified path
    ///
    pub fn overlaps_path<P: BezierPath<Point=Coord2>>(&self, path: &Vec<P>) -> RegionPath {
        // Create a GraphPath from this path
        let mut animation_path = GraphPath::new();

        for our_component in self.path.iter() {
            let component_path  = GraphPath::from_path(our_component, PathLabel(0));

            animation_path      = animation_path.merge(component_path);
        }

        animation_path.self_collide(0.01);

        // Create a GraphPath from the region path
        let mut region_path     = GraphPath::new();

        for their_component in path.iter() {
            let component_path  = GraphPath::from_path(their_component, PathLabel(1));

            region_path         = region_path.merge(component_path);
        }

        region_path.self_collide(0.01);

        // Perform an intersection
        let mut graph_path      = animation_path.collide(region_path, 0.01);
        graph_path.round(0.01);
        graph_path.set_exterior_by_intersecting();
        graph_path.heal_exterior_gaps();

        // Return as a region path
        RegionPath::new(graph_path)
    }

    ///
    /// Returns true if this path is overlaps the specified region
    ///
    pub fn overlaps_region<R: AnimationRegion>(&self, region: &R, time: Duration) -> RegionPath {
        self.overlaps_path(&region.region(time))
    }
}

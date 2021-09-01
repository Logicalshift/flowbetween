use crate::path::*;

use flo_curves::*;
use flo_curves::bezier::path::*;

use std::cmp::{Ordering};

///
/// Cached values for an animation layer
///
pub struct AnimationLayerCache {
    /// Bounding boxes for all of the paths in the drawing with their index, ordered by their minimum x-coordinate
    pub (crate) bounding_boxes: Option<Vec<(usize, Bounds<Coord2>)>>
}

impl AnimationLayerCache {
    ///
    /// Creates a new empty cache
    ///
    pub fn new() -> AnimationLayerCache {
        AnimationLayerCache {
            bounding_boxes: None
        }
    }

    ///
    /// Resets the cached values
    ///
    pub fn flush(&mut self) {
        self.bounding_boxes = None;
    }

    ///
    /// Calculates the bounding boxes for the specified drawing
    ///
    pub fn calculate_bounding_boxes(&mut self, drawing: &Vec<AnimationPath>) {
        // Calculate the bounding boxes for each path
        let bounding_boxes = drawing.iter().map(|path| {
            let components      = PathComponent::from_path(&path);
            let bounding_boxes  = components.into_iter().map(|component| component.bounding_box());
            let bbox            = bounding_boxes.fold(Bounds::empty(), |a, b| a.union_bounds(b));

            bbox
        });

        // Order by minimum x coordinate
        let mut bounding_boxes  = bounding_boxes.enumerate().collect::<Vec<_>>();
        bounding_boxes.sort_by(|&(_, a_bounds), &(_, b_bounds)| {
            a_bounds.min().x().partial_cmp(&b_bounds.min().x()).unwrap_or(Ordering::Equal)
        });

        // Store the bounding boxes for the drawing
        self.bounding_boxes     = Some(bounding_boxes);
    }
}

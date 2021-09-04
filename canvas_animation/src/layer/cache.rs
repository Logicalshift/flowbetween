use crate::path::*;
use crate::region::*;

use flo_curves::*;
use flo_curves::bezier::path::*;

use itertools::*;

use std::cmp::{Ordering};
use std::sync::*;
use std::time::{Duration};

///
/// Cached values for an animation layer
///
pub struct AnimationLayerCache {
    /// Bounding boxes for all of the paths in the drawing with their index, ordered by their minimum x-coordinate
    pub (crate) bounding_boxes: Option<Vec<(usize, Bounds<Coord2>)>>,

    /// The frames where the drawing changes (so we need to recalculate regions)
    pub (crate) drawing_times: Option<Vec<Duration>>,

    /// Bounding boxes for the animated regions, ordered by their minimum x-coordinate
    pub (crate) region_bounding_boxes: Option<Vec<(usize, Bounds<Coord2>)>>
}

impl AnimationLayerCache {
    ///
    /// Creates a new empty cache
    ///
    pub fn new() -> AnimationLayerCache {
        AnimationLayerCache {
            bounding_boxes:         None,
            drawing_times:          None,
            region_bounding_boxes:  None
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

    ///
    /// Fills in the cache of times when we need to recalculate region contents due to extra drawing appearing in the canvas
    ///
    pub fn calculate_drawing_times(&mut self, drawing: &Vec<AnimationPath>) {
        let drawing_times = drawing.iter().map(|path| path.appearance_time)
            .unique()
            .sorted()
            .collect::<Vec<_>>();

        self.drawing_times = Some(drawing_times);
    }

    ///
    /// Calculates the bounding boxes for the animation regions
    ///
    pub fn calculate_region_bounding_boxes(&mut self, drawing: &Vec<AnimationPath>, regions: &Vec<Arc<dyn AnimationRegion>>) {
        // Need the bounding boxes at every point that the drawing is altered
        if self.drawing_times.is_none() {
            self.calculate_drawing_times(drawing);
        }

        let drawing_times = self.drawing_times.as_ref().unwrap();

        // TODO: calculate the bounding boxes at all of the drawing times
        let bounding_boxes = regions.iter().map(|region| {
            let region_paths        = region.region(Duration::from_millis(0));
            let bounding_boxes      = region_paths.into_iter().map(|path| path.bounding_box());
            let bbox                = bounding_boxes.fold(Bounds::empty(), |a, b| a.union_bounds(b));

            bbox
        });

        let mut bounding_boxes = bounding_boxes.enumerate().collect::<Vec<_>>();
        bounding_boxes.sort_by(|&(_, a_bounds), &(_, b_bounds)| {
            a_bounds.min().x().partial_cmp(&b_bounds.min().x()).unwrap_or(Ordering::Equal)
        });

        self.region_bounding_boxes  = Some(bounding_boxes);    
    }
}

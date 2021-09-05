use crate::path::*;
use crate::region::*;

use flo_curves::*;
use flo_curves::bezier::path::*;

use itertools::*;

use std::cmp::{Ordering};
use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Represents the bounding box of a possibly overlapping region in the animation layer
///
pub struct RegionBounds {
    /// The ID of the overlapping regions that these bounds are for
    pub regions: Vec<RegionId>,

    /// The perimeter of the bounding box as a path
    pub perimeter: Arc<Vec<SimpleBezierPath>>,

    /// The outer bounds of the bounding box
    pub bounds: Bounds<Coord2>
}

impl RegionBounds {
    ///
    /// Returns the minimum point of this bounding box
    ///
    #[inline] pub fn min(&self) -> Coord2 { self.bounds.min() }
}

///
/// Cached values for an animation layer
///
pub struct AnimationLayerCache {
    /// Bounding boxes for all of the paths in the drawing with their index, ordered by their minimum x-coordinate
    pub (crate) bounding_boxes: Option<Vec<(usize, Bounds<Coord2>)>>,

    /// The frames where the drawing changes (so we need to recalculate regions), in order
    pub (crate) drawing_times: Option<Vec<Duration>>,

    /// Bounding boxes for the animated regions, ordered by their time and minimum x-coordinate
    pub (crate) region_bounding_boxes: Option<Vec<(Duration, Vec<RegionBounds>)>>
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

        // Compute the bounding boxes at each of these times
        let bounding_boxes_by_time = drawing_times.iter()
            .map(|time| {
                // The regions themselves might overlap: reduce to a set of non-overlapping regions
                let region_collection           = collect_regions(regions.iter(), *time);
                let non_overlapping_regions     = intersect_regions(region_collection, 0.01);

                // Work out the bounding boxes for each region
                let bounding_boxes      = non_overlapping_regions.into_iter().map(|(overlapping_region_ids, perimeter_path)| {
                    // Combine the bounding boxes of the paths that make up the perimeter to a single path
                    let bounding_boxes      = perimeter_path.iter().map(|path| path.bounding_box());
                    let bbox                = bounding_boxes.fold(Bounds::empty(), |a, b| a.union_bounds(b));

                    // Store the resulting region in a RegionBounds structure
                    RegionBounds {
                        regions:    overlapping_region_ids,
                        perimeter:  Arc::new(perimeter_path),
                        bounds:     bbox
                    }
                });

                let mut bounding_boxes  = bounding_boxes.collect::<Vec<_>>();
                bounding_boxes.sort_by(|a_bounds, b_bounds| {
                    a_bounds.min().x().partial_cmp(&b_bounds.min().x()).unwrap_or(Ordering::Equal)
                });

                (*time, bounding_boxes)
            });

        self.region_bounding_boxes  = Some(bounding_boxes_by_time.collect());    
    }

    ///
    /// Cuts a drawing consisting of paths into regions
    ///
    /// For the moment, we divide paths at region boundaries: a future enhancement could be to make it possible to
    /// make cutting the paths optional, for instance to make it so only a particular set of points within a path
    /// actually move. This is currently not supported as a single path could overlap potentially many regions and
    /// combining the effects to generate an output path makes things very complicated, particularly where regions 
    /// want to do things like further cut up the path rather than just rearrange the points.
    ///
    pub fn cut_drawing_into_regions(&mut self, drawing: &Vec<AnimationPath>, regions: &Vec<Arc<dyn AnimationRegion>>) {
        // Ensure all of the prerequisite caches are filled
        if self.bounding_boxes.is_none()        { self.calculate_bounding_boxes(drawing); }
        if self.drawing_times.is_none()         { self.calculate_drawing_times(drawing); }
        if self.region_bounding_boxes.is_none() { self.calculate_region_bounding_boxes(drawing, regions); }

        let drawing_bounding_boxes  = if let Some(bounding_boxes)        = &self.bounding_boxes        { bounding_boxes }        else { return; };
        let drawing_times           = if let Some(drawing_times)         = &self.drawing_times         { drawing_times }         else { return; };
        let region_bounding_boxes   = if let Some(region_bounding_boxes) = &self.region_bounding_boxes { region_bounding_boxes } else { return; };

        // Nothing to do if there are no drawing times
        if drawing_times.len() == 0 { return; }

        // Set up the initial region bounds for the earliest time.
        let earliest_drawing_time   = drawing_times[0];
        let active_regions          = region_bounding_boxes.iter()
            .map(|(when, region_bounds)| {
                (*when, region_bounds)
            })
            .collect::<HashMap<_, _>>();

        // Borrow the earlier active region
        let mut current_time        = earliest_drawing_time;
        let mut current_regions     = active_regions.get(&current_time).unwrap();

        // Process the drawing instructions from left-to-right
        for path in drawing.iter() {
            
        }
    }
}

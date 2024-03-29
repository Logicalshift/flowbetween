use super::path_index::*;

use crate::path::*;
use crate::region::*;

use flo_curves::*;
use flo_curves::bezier::path::*;
use flo_canvas::*;

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
    /// Bounding boxes for all of the paths in the drawing with their index, ordered by their path index
    pub (crate) drawing_bounding_boxes: Option<Vec<(PathIndex, Bounds<Coord2>)>>,

    /// The frames where the drawing changes (so we need to recalculate regions), in order
    pub (crate) drawing_times: Option<Vec<Duration>>,

    /// Bounding boxes for the animated regions, ordered by their time and minimum x-coordinate
    pub (crate) region_bounding_boxes: Option<Vec<(Duration, Vec<RegionBounds>)>>,

    /// The paths contained within each set of regions, in path index order (the empty set for paths that are not in any region)
    pub (crate) paths_for_region: Option<HashMap<Vec<RegionId>, Arc<AnimationRegionContent>>>
}

impl AnimationLayerCache {
    ///
    /// Creates a new empty cache
    ///
    pub fn new() -> AnimationLayerCache {
        AnimationLayerCache {
            drawing_bounding_boxes: None,
            drawing_times:          None,
            region_bounding_boxes:  None,
            paths_for_region:       None
        }
    }

    ///
    /// Resets the cached values
    ///
    pub fn flush(&mut self) {
        self.drawing_bounding_boxes = None;
        self.drawing_times          = None;
        self.region_bounding_boxes  = None;
        self.paths_for_region       = None;
    }

    ///
    /// Calculates the bounding boxes for the specified drawing
    ///
    pub fn calculate_bounding_boxes(&mut self, drawing: &Vec<AnimationPath>) {
        // Calculate the bounding boxes for each path
        let bounding_boxes = drawing.iter().map(|path| {
            let components      = &path.path;
            let bounding_boxes  = components.iter().map(|component| component.bounding_box());
            let bbox            = bounding_boxes.fold(Bounds::empty(), |a, b| a.union_bounds(b));

            bbox
        });

        // Order by minimum x coordinate
        let mut bounding_boxes  = bounding_boxes.enumerate().map(|(idx, bbox)| (PathIndex(idx), bbox)).collect::<Vec<_>>();
        bounding_boxes.sort_by(|&(a_idx, _a_bounds), &(b_idx, _b_bounds)| {
            a_idx.cmp(&b_idx)
        });

        // Store the bounding boxes for the drawing
        self.drawing_bounding_boxes = Some(bounding_boxes);
    }

    ///
    /// Fills in the cache of times when we need to recalculate region contents due to extra drawing appearing in the canvas
    ///
    pub fn calculate_drawing_times(&mut self, drawing: &Vec<AnimationPath>) {
        let drawing_times = drawing.iter().map(|path| path.appearance_time)
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
        if self.drawing_bounding_boxes.is_none()    { self.calculate_bounding_boxes(drawing); }
        if self.drawing_times.is_none()             { self.calculate_drawing_times(drawing); }
        if self.region_bounding_boxes.is_none()     { self.calculate_region_bounding_boxes(drawing, regions); }

        let drawing_bounding_boxes  = if let Some(drawing_bounding_boxes) = &self.drawing_bounding_boxes { drawing_bounding_boxes } else { return; };
        let drawing_times           = if let Some(drawing_times)          = &self.drawing_times          { drawing_times }          else { return; };
        let region_bounding_boxes   = if let Some(region_bounding_boxes)  = &self.region_bounding_boxes  { region_bounding_boxes }  else { return; };

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

        // Store the paths that are in each region
        let mut cut_paths           = HashMap::new();

        // Process the drawing instructions in path index order
        for path_bounds in drawing_bounding_boxes.iter() {
            // Initially the whole path is remaining
            let (path_idx, remaining_bounds)    = *path_bounds;
            let mut remaining_path              = drawing[path_idx.idx()].clone();

            // Change the current region depending on the time that this path appears
            let drawing_time = drawing_times[path_idx.idx()];
            if drawing_time != current_time {
                current_regions = active_regions.get(&drawing_time).unwrap();
                current_time    = drawing_time;
            }

            // Match up against the bounds of the regions at the time that the path appears in the animation
            for region_perimeter in current_regions.iter() {
                // Ignore this region if the path doesn't overlap it
                if !region_perimeter.bounds.overlaps(&remaining_bounds) {
                    continue;
                }

                // Test to see how the path and the region overlaps
                let mut overlap = remaining_path.overlaps_path(&*region_perimeter.perimeter);

                match overlap.region_type() {
                    // The path is entirely outside of the region, just move to the next region for testing
                    PathRegionType::OutsideRegion       => { continue; }

                    // The path is entirely inside the region: add to the list of paths affected by this region and stop (there's no path left at this point)
                    PathRegionType::InsideRegion        => { 
                        // Store the entire path in the result
                        cut_paths.entry(region_perimeter.regions.clone())
                            .or_insert_with(|| vec![])
                            .push(remaining_path.clone());

                        // There's no remaining path
                        remaining_path = remaining_path.with_path(Arc::new(vec![]));
                        break; 
                    }

                    // The region needs to cut out part of the path
                    PathRegionType::IntersectsRegion    |
                    PathRegionType::EnclosesRegion      => {
                        // Cut out the part in the region, and leave the remainder in remaining_path for other regions
                        // (Regions have been processed so they don't overlap here so the cut out path will end up in the other regions)
                        let inside_path = remaining_path.with_path(Arc::new(overlap.path_inside()));
                        remaining_path  = remaining_path.with_path(Arc::new(overlap.path_outside()));

                        // Store the inside path in the result
                        cut_paths.entry(region_perimeter.regions.clone())
                            .or_insert_with(|| vec![])
                            .push(inside_path);
                    }
                }
            }

            // If there's any remaining path, add to the paths outside of any region
            if remaining_path.path.len() > 0 {
                cut_paths.entry(vec![])
                    .or_insert_with(|| vec![])
                    .push(remaining_path);
            }
        }

        // Store the result in the cache
        let cut_paths           = cut_paths.into_iter()
            .map(|(region_id, paths)| (region_id, Arc::new(AnimationRegionContent::from_paths(paths))))
            .collect();
        self.paths_for_region   = Some(cut_paths);
    }

    ///
    /// Uses the contents of this cache and a list of regions to render the layer at a particular time
    ///
    pub fn render_at_time<Context: GraphicsContext+?Sized>(&mut self, time: Duration, regions: &Vec<Arc<dyn AnimationRegion>>, ctxt: &mut Context) {
        // Fetch the regions from the cache
        let region_paths    = if let Some(paths) = self.paths_for_region.as_ref() { paths } else { return; };

        // Order the regions so that the empty region is first, followed by the other regions in order
        let ordered_regions = region_paths.keys()
            .sorted_by(|region_a, region_b| {
                if region_a.len() < region_b.len() {
                    Ordering::Less
                } else if region_a.len() > region_b.len() {
                    Ordering::Greater
                } else {
                    region_a.cmp(region_b)
                }
            });

        // Process and draw the regions in order
        // TODO: generate the cached versions of the region animation
        for region_ids in ordered_regions {
            // Get the paths for this region
            let paths       = region_paths.get(region_ids).unwrap();
            let mut content = Arc::clone(paths);

            for region_id in region_ids.iter() {
                // Apply the animation in this region
                let region      = &regions[region_id.0];
                let new_paths   = region.animate(content, time);
                content         = new_paths;
            }

            // Add the content for this region to the rendering
            for drawing in content.to_drawing(time) { ctxt.draw(drawing); }
        }
    }
}

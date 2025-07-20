//!
//! 'Blobland'
//!
//! We show the tools and other UI elements as circles that can be merged together as 'clovers'. When items
//! get near each other we want to show if they can attach (are attracted to each other), or are incompatible
//! (repel each other). For this, we use this 'blobland' renderer, which computes what to render using a distance
//! field with a 'sea level'. Things that attract each other raise the sea level around them, and things that
//! repel lower the sea level.
//!
//! Another effect we want to generate is a 'tear off' effect when dragging out items from the dock (dragging from
//! the dock doesn't undock or rearrange things but creates copies)
//!

use super::ui_path::*;

use flo_curves::*;
use flo_curves::geo::*;
use flo_curves::bezier::*;
use flo_curves::bezier::rasterize::*;
use flo_curves::bezier::vectorize::*;
use flo_draw::canvas::*;

use smallvec::*;
use once_cell::sync::{Lazy};

use std::collections::*;
use std::ops::{Range};
use std::f64;
use std::sync::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Amount to divide the canvas size by for the blob contour
const BLOB_CONTOUR_SIZE_RATIO: f64 = 4.0;

///
/// Ways that two blobs can interact with each other
///
pub enum BlobInteraction {
    /// Blobs have no effect on each other
    None,

    /// This blob is attracted to the other blob
    Attract,

    /// This blob is repelled from the other blob
    Repel,
}

///
/// An ID for a blob in the blobland
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BlobId(usize);

///
/// A circular 'Blob' that lives in the blobland (not a Binary Large OBject: an actual blob)
///
#[derive(Debug)]
pub struct Blob {
    /// An identifier for this blob
    id: BlobId,

    /// The position of the center of this blob
    pos: UiPoint,

    /// The radius, in pixels, of this blob
    radius: f64,

    /// The radius, in pixels, of the part of this blob that's above 'sea level' by default
    island_radius: f64,

    /// The distance that the points in this blob prefer to be from one another
    point_distance: f64,

    /// The points that represent the outline of this blob
    points: Vec<BlobPoint>,
}

///
/// Represents a blobland
///
pub struct BlobLand {
    /// The blobs in this land
    blobs: HashMap<BlobId, Blob>,

    /// The blobs on each canvas line
    blobs_on_line: Mutex<Option<Vec<Vec<BlobId>>>>,

    /// The size of the canvas
    canvas_size: ContourSize,

    /// Time that wasn't accounted for in the last simulation step
    extra_time: f64,

    /// Blobs that are close enough to interact
    interacting_blobs: HashMap<BlobId, Vec<BlobId>>,
}

///
/// Each blob consists of a series of points, on which various forces act to create the animations
///
/// Blobs are rendered by fitting these points to curves
///
#[derive(Clone, Copy, Debug)]
struct BlobPoint {
    /// Where this point is located
    pos: UiPoint,

    /// The offset from the center of the blob to where this point prefers to be located
    home_offset: UiPoint,

    /// Velocity, in points per second
    velocity: UiPoint,
}

impl BlobId {
    ///
    /// Creates a new, unique (within this process), blob ID
    ///
    pub fn new() -> BlobId {
        pub static NEXT_ID: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::from(0));

        let blob_id = (*NEXT_ID).fetch_add(1, Ordering::Relaxed);

        BlobId(blob_id)
    }
}

impl Blob {
    ///
    /// Creates a new blob
    ///
    pub fn new(pos: UiPoint, radius: f64, island_radius: f64) -> Blob {
        // The points are initially around the center position
        let num_points      = 16;
        let circumference   = 2.0 * island_radius * f64::consts::PI;
        let points          = (0..num_points).into_iter()
            .map(|point_num| {
                let angle       = (2.0*f64::consts::PI / (num_points as f64)) * (point_num as f64);
                let x_offset    = angle.sin() * island_radius;
                let y_offset    = angle.cos() * island_radius;
                let point_pos   = UiPoint(pos.0 + x_offset, pos.1 + y_offset);

                BlobPoint {
                    pos:            point_pos,
                    home_offset:    UiPoint(x_offset, y_offset),
                    velocity:       UiPoint(0.0, 0.0),
                }
            }).collect();

        // Create the blob
        Blob {
            id:             BlobId::new(),
            pos:            pos,
            radius:         radius,
            island_radius:  island_radius,
            point_distance: circumference / (num_points as f64),
            points:         points,
        }
    }

    ///
    /// Retrieves the ID for this blob
    ///
    pub fn id(&self) -> BlobId {
        self.id
    }

    ///
    /// Returns true if this blob is interacting with another one (is close enough to repel or attract it)
    ///
    pub fn is_interacting_with(&self, other_blob: &Blob) -> bool {
        let distance = self.pos.distance_to(&other_blob.pos);

        if distance < self.radius*2.0 && distance < other_blob.radius*2.0 {
            true
        } else {
            false
        }
    }

    ///
    /// Creates a path representing this blob in the graphics context
    ///
    pub fn render_path(&self, gc: &mut impl GraphicsContext) {
        // Fit against the points
        let points      = self.points.iter().map(|point| point.pos).chain(self.points.get(0).map(|point| point.pos)).collect::<Vec<_>>();
        let fit_curves  = fit_curve::<Curve<UiPoint>>(&points, 0.1);

        gc.new_path();

        if let Some(fit_curves) = fit_curves {
            // Create a path from the points
            gc.move_to(fit_curves[0].start_point().0 as _, fit_curves[0].start_point().1 as _);
            for curve in fit_curves.into_iter() {
                gc.bezier_curve(&curve);
            }
            gc.close_path();
        }
    }
}

impl BlobLand {
    ///
    /// Creates an empty blobland
    ///
    pub fn empty() -> BlobLand {
        BlobLand {
            blobs:              HashMap::new(),
            blobs_on_line:      Mutex::new(None),
            canvas_size:        ContourSize(0, 0),
            extra_time:         0.0,
            interacting_blobs:  HashMap::new(),
        }
    }

    ///
    /// Adds a blob to this land
    ///
    pub fn add_blob(&mut self, blob: Blob) {
        self.blobs.insert(blob.id(), blob);
        *self.blobs_on_line.lock().unwrap() = None;
    }

    ///
    /// Sets the position of an existing blob
    ///
    pub fn move_blob(&mut self, blob_id: BlobId, new_pos: UiPoint) {
        if let Some(blob) = self.blobs.get_mut(&blob_id) {
            let offset = new_pos - blob.pos;
            blob.pos = new_pos;

            for point in blob.points.iter_mut() {
                point.pos = point.pos + offset;
            }

            *self.blobs_on_line.lock().unwrap() = None;
        }
    }

    ///
    /// Updates the canvas size that the BlobLand will be rendered over
    ///
    pub fn set_canvas_size(&mut self, size: (f64, f64)) {
        // The contour is a smaller size than the canvas (this is because we use the distance field to generate a vector and don't need the full resolution)
        let width   = (size.0/BLOB_CONTOUR_SIZE_RATIO).ceil();
        let height  = (size.1/BLOB_CONTOUR_SIZE_RATIO).ceil();

        self.canvas_size = ContourSize(width as _, height as _);
    }

    ///
    /// Retrieves the blob with the specified ID
    ///
    #[inline]
    pub fn blob(&self, blob_id: BlobId) -> Option<&Blob> {
        self.blobs.get(&blob_id)
    }

    ///
    /// Returns the IDs of the blobs that are at a particular y-position
    ///
    #[inline]
    pub fn blobs_on_line(&self, ypos: usize) -> Vec<BlobId> {
        if let Some(blobs) = &*self.blobs_on_line.lock().unwrap() {
            // Just retrieve from the cache
            blobs.get(ypos)
                .cloned()
                .unwrap_or_else(|| vec![])
        } else {
            // Fill the blob cache and try again
            self.fill_blobs_on_line_cache();

            // Retry the fetch with the filled cache
            self.blobs_on_line.lock().unwrap().as_ref()
                .and_then(|blobs_on_line| blobs_on_line.get(ypos))
                .cloned()
                .unwrap_or_else(|| vec![])
        }
    }

    ///
    /// Runs the simulation for the specified time
    ///
    /// Returns true if the simulation should go to sleep (no more simulations needed until the blobland is disturbed by something)
    ///
    pub fn simulate(&mut self, delta_t: f64) -> bool {
        // Account for the extra time, but only simulate up to MAX_TICKS total time
        let mut delta_t = (delta_t + self.extra_time).min(MAX_TICKS);

        // Create the blobs, sorted by y position (we use this to discover which blobs are interacting later on)
        // We don't move the blobs per tick so the interacting set doesn't change here
        let mut sorted_blobs = self.blobs.keys().copied().collect::<Vec<_>>();
        sorted_blobs.sort_by(|a, b| {
            let a_start = self.blobs.get(a).map(|blob| blob.pos.1 - blob.radius).unwrap_or(0.0);
            let b_start = self.blobs.get(b).map(|blob| blob.pos.1 - blob.radius).unwrap_or(0.0);

            a_start.total_cmp(&b_start)
        });

        // Sweep the blobs to discover which ones are interacting
        let mut active_blobs = vec![];
        let mut interacting_blobs = HashMap::new();

        for blob_id in sorted_blobs.into_iter() {
            // Fetch the next blob to process and its position
            let blob    = if let Some(blob) = self.blobs.get(&blob_id) { blob } else { continue; };
            let min_y   = blob.pos.1 - blob.radius;
            let max_y   = blob.pos.1 + blob.radius;

            // Remove any blobs from the active list that can't be interacting with this blob
            active_blobs.retain(|active_blob_id| {
                if let Some(active_blob) = self.blobs.get(active_blob_id) {
                    let active_max_y = active_blob.pos.1 + active_blob.radius;
                    if active_max_y < min_y {
                        // Blob finishes before the new blob starts
                        false
                    } else {
                        // (assuming the blobs are properly ordered)
                        true
                    }
                } else {
                    // Blob doesn't exist in the hashset for some reason
                    unreachable!(); // Because the hashset doesn't change
                    false
                }
            });

            // Check the new blob for any interactions (blobs whose outer radiuses overlap), and add to the interaction set if there are any
            let new_blob = self.blobs.get(&blob_id).unwrap();

            for other_blob_id in active_blobs.iter().copied() {
                let other_blob = self.blobs.get(&other_blob_id).unwrap();

                if new_blob.is_interacting_with(other_blob) {
                    interacting_blobs.entry(blob_id).or_insert_with(|| vec![]).push(other_blob_id);
                    interacting_blobs.entry(other_blob_id).or_insert_with(|| vec![]).push(blob_id);
                }
            }

            // The blob we just picked always becomes part of the active set
            active_blobs.push(blob_id);
        }

        // Run the simulation for each tick
        let blob_ids = self.blobs.keys().copied().collect::<Vec<_>>();

        while delta_t >= TICK {
            for blob_id in blob_ids.iter().copied() {
                // Create a list of the updated points for this blob
                let interacting_with    = interacting_blobs.get(&blob_id).into_iter()
                    .flatten()
                    .flat_map(|blob_id| self.blobs.get(blob_id))
                    .map(|blob| blob.pos)
                    .collect();
                let blob                = self.blobs.get_mut(&blob_id).unwrap();
                let mut updated_points  = Vec::with_capacity(blob.points.len());

                // Simulate each point
                for idx in 0..blob.points.len() {
                    // Previous and next points for the simulation
                    let prev_idx    = if idx == 0 { blob.points.len()-1 } else { idx-1 };
                    let next_idx    = if idx >= blob.points.len()-1 { 0 } else { idx+1 };
                    let this_point  = &blob.points[idx];
                    let prev_point  = &blob.points[prev_idx];
                    let next_point  = &blob.points[next_idx];

                    // Run the simulation for this point
                    let updated_point = this_point.simulate_tick(blob.point_distance, blob.pos, blob.radius, next_point, prev_point, &interacting_with, &vec![]);
                    //let updated_point = this_point.simulate_tick(blob.point_distance, blob.pos, blob.radius, next_point, prev_point, &vec![], &interacting_with);
                    updated_points.push(updated_point);
                }

                // Update the points in the blob
                blob.points = updated_points;
            }

            // Move forward a tick
            delta_t -= TICK;
        }

        // Any 'left over' time should be accounted for in the next simulation step
        self.extra_time = delta_t;

        // Store what we've found as the interacting blobs
        self.interacting_blobs = interacting_blobs;

        false
    }

    ///
    /// Renders the blobland to a graphics context
    ///
    pub fn render(&self, gc: &mut impl GraphicsContext) {
        for blob in self.blobs.values() {
            // TODO: for interacting blobs, add the resulting paths
            blob.render_path(gc);
            gc.fill();
            gc.stroke();
        }
    }

    ///
    /// Renders a set of interacting blobs
    ///
    fn render_interacting(&self, blobs: Vec<BlobId>) {
        // Rather than rendering each blob as a plain circle, we instead create a set of points that are attracted to or repelled from the center 
        // of each other blob, then add them together to create a final path
    }

    ///
    /// Fills the cache of blobs on lines (we use this when generating the contour as a fast way to find the blobs we need to calculate for each line)
    ///
    fn fill_blobs_on_line_cache(&self) {
        // Sort the blob IDs by start y-positions
        let mut sorted_blobs = self.blobs.keys().copied().collect::<Vec<_>>();
        sorted_blobs.sort_by(|a, b| {
            let a_start = self.blobs.get(a).map(|blob| blob.pos.1 - blob.radius).unwrap_or(0.0);
            let b_start = self.blobs.get(b).map(|blob| blob.pos.1 - blob.radius).unwrap_or(0.0);

            a_start.total_cmp(&b_start)
        });

        // Sweep the sorted blobs to generate the blobs on each line
        let mut sorted_blobs_iter   = sorted_blobs.into_iter();
        let mut next_blob           = sorted_blobs_iter.next();
        let mut active_blobs        = vec![];
        let mut blobs_on_line       = Vec::with_capacity(self.canvas_size.1);

        for y_pos in 0..(self.canvas_size.1) {
            let y_start = y_pos as f64;
            let y_end   = (y_pos as f64) + 1.0;

            // Remove any blobs that end before this y position (ie, which are no longer being covered by this range)
            active_blobs.retain(|blob_id| {
                let blob = self.blobs.get(blob_id);
                if let Some(blob) = blob {
                    if blob.pos.1 + blob.radius < y_end {
                        false
                    } else {
                        true
                    }
                } else {
                    unreachable!();     // because the hashset hasn't changed
                    false
                }
            });

            // Add any new blobs to the active set
            while let Some(maybe_new_blob) = next_blob {
                if let Some(blob) = self.blobs.get(&maybe_new_blob) {
                    if blob.pos.1 - blob.radius <= y_start {
                        // This blob is included in the active set (starts before the current y position)
                        active_blobs.push(maybe_new_blob);

                        // Keep trying to add more blobs after this one
                        next_blob = sorted_blobs_iter.next();
                    } else {
                        // This blob is not yet active
                        next_blob = Some(maybe_new_blob);
                        break;
                    }
                } else {
                    unreachable!();     // because the hashset hasn't changed

                    // Just move to the next blob if it's somehow disappeared (should be unreachable)
                    next_blob = sorted_blobs_iter.next();
                }
            }

            // Store the active set in the blobs on line list
            blobs_on_line.push(active_blobs.clone());
        }

        // Store the new set of blobs on
        *self.blobs_on_line.lock().unwrap() = Some(blobs_on_line);
    }
}

/// Length of time per simulation tick
const TICK: f64             = 1.0 / 60.0;

/// Maximum number of ticks to run in one simulation pass (if the simulation gets delayed for longer than this, the time is 'lost')
const MAX_TICKS: f64        = 30.0;

/// Friction
const FRICTION: f64         = 0.91;

/// Force used to push the points into a circular shape
const RADIUS_FORCE: f64     = 64.0;

/// Force used to push the points towards or away from their neighbors
const NEIGHBOR_FORCE: f64   = 64.0;

/// Force used to push points away from repulsors
const REPULSOR_FORCE: f64   = 2048.0;

/// Force used to pull points towards attractors
const ATTRACTOR_FORCE: f64  = 2048.0;

///
/// Calculates the spring between point_a and point_b, with a natural length of 'length'
///
/// Force is in the direction a -> b
///
#[inline]
fn spring_force(point_a: UiPoint, point_b: UiPoint, length: f64, force_factor: f64) -> UiPoint {
    // This is a fairly unphysical algorithm
    let offset      = point_a - point_b;
    let distance    = offset.dot(&offset).sqrt();
    let tension     = distance - length;
    let force       = tension * force_factor;
    let unit_offset = offset.to_unit_vector();

    unit_offset * -force
}

///
/// Calculates the 'gravity force' between point_a and point_b (a force that gets stronger the closer the two points are, goes to 0 at max_distance)
///
#[inline]
fn gravity_force(point_a: UiPoint, point_b: UiPoint, max_distance: f64, force_factor: f64) -> UiPoint {
    // ... not really gravity as this force varies linearly
    let offset      = point_b - point_a;
    let distance    = offset.dot(&offset).sqrt();
    let force       = if distance < max_distance { force_factor * (1.0-(distance/max_distance)) } else { 0.0 };
    let unit_offset = offset.to_unit_vector();

    unit_offset * force
}

impl BlobPoint {
    ///
    /// Runs a simulation on this point for a single tick, returning the updated point
    ///
    fn simulate_tick(&self, point_distance: f64, center: UiPoint, radius: f64, next_point: &BlobPoint, previous_point: &BlobPoint, attractors: &Vec<UiPoint>, repulsors: &Vec<UiPoint>) -> BlobPoint {
        // Take the values from inside this 
        let mut pos         = self.pos;
        let mut velocity    = self.velocity;
        let home_offset     = self.home_offset;

        // Friction (1 tick)
        velocity = velocity * FRICTION;

        // Push towards the origin point (more force the further away it is)
        let home_pos        = center + home_offset;
        let home_distance   = home_pos - pos;
        let home_force      = home_distance * RADIUS_FORCE;

        velocity = velocity + home_force * TICK;

        // Points are attached to each other with springs
        velocity = velocity + spring_force(pos, next_point.pos, point_distance, NEIGHBOR_FORCE * TICK);
        velocity = velocity + spring_force(pos, previous_point.pos, point_distance, NEIGHBOR_FORCE * TICK);

        // Process the attractors and repulsors
        for attractor in attractors.iter() {
            velocity = velocity + gravity_force(pos, *attractor, radius, ATTRACTOR_FORCE * TICK);
        }

        for repulsor in repulsors.iter() {
            velocity = velocity - gravity_force(pos, *repulsor, radius, REPULSOR_FORCE * TICK);
        }

        // Move the point
        pos = pos + velocity * TICK;

        BlobPoint { pos, home_offset, velocity }
    }
}

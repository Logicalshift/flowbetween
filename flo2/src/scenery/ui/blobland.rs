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
//! Technically the items can be any shape but we prefer to use circles.
//!

use flo_curves::bezier::rasterize::*;
use flo_curves::bezier::vectorize::*;

use smallvec::*;
use once_cell::sync::{Lazy};

use std::collections::*;
use std::ops::{Range};
use std::sync::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Amount to divide the canvas size by for the blob contour
const BLOB_CONTOUR_SIZE_RATIO: f64 = 4.0;

///
/// An ID for a blob in the blobland
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BlobId(usize);

///
/// A circular 'Blob' that lives in the blobland (not a Binary Large OBject: an actual blob)
///
#[derive(Clone, Debug)]
pub struct Blob {
    /// An identifier for this blob
    id: BlobId,

    /// The position of the center of this blob
    pos: (f64, f64),

    /// The radius, in pixels, of this blob
    radius: f64,

    /// The radius, in pixels, of the part of this blob that's above 'sea level' by default
    island_radius: f64,
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
    pub fn new(pos: (f64, f64), radius: f64, island_radius: f64) -> Blob {
        Blob {
            id:             BlobId::new(),
            pos:            pos,
            radius:         radius,
            island_radius:  island_radius,
        }
    }

    ///
    /// Retrieves the ID for this blob
    ///
    pub fn id(&self) -> BlobId {
        self.id
    }
}

impl BlobLand {
    ///
    /// Creates an empty blobland
    ///
    pub fn empty() -> BlobLand {
        BlobLand {
            blobs:          HashMap::new(),
            blobs_on_line:  Mutex::new(None),
            canvas_size:    ContourSize(0, 0),
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
    pub fn move_blob(&mut self, blob_id: BlobId, new_pos: (f64, f64)) {
        if let Some(blob) = self.blobs.get_mut(&blob_id) {
            blob.pos = new_pos;
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
    /// Fills the cache of blobs on lines
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

impl SampledContour for BlobLand {
    ///
    /// The size of this contour
    ///
    #[inline]
    fn contour_size(&self) -> ContourSize {
        self.canvas_size
    }

    ///
    /// Given a y coordinate returns ranges indicating the filled pixels on that line
    ///
    /// The ranges must be provided in ascending order, and must also not overlap.
    ///
    fn intercepts_on_line(&self, y: f64) -> SmallVec<[Range<f64>; 4]> {
        todo!()
    }
}

impl SampledSignedDistanceField for BlobLand {
    /// A type that can represent the edge contour for this distance field (see `ContourFromDistanceField` for a basic implementation)
    type Contour = Self;

    ///
    /// The size of this distance field
    ///
    fn field_size(&self) -> ContourSize {
        self.canvas_size
    }

    ///
    /// Returns the distance to the nearest edge of the specified point (a negative value if the point is inside the shape)
    ///
    fn distance_at_point(&self, pos: ContourPosition) -> f64 {
        todo!()
    }

    ///
    /// Returns an edge contour for this distance field
    ///
    fn as_contour<'a>(&'a self) -> &'a Self::Contour {
        self
    }
}
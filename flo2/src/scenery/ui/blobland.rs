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

    /// The blobs, sorted into y cordinates
    y_order: Mutex<Option<Vec<BlobId>>>,
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
            blobs:      HashMap::new(),
            y_order:    Mutex::new(None),
        }
    }

    ///
    /// Adds a blob to this land
    ///
    pub fn add_blob(&mut self, blob: Blob) {
        self.blobs.insert(blob.id(), blob);
        *self.y_order.lock().unwrap() = None;
    }

    ///
    /// Sets the position of an existing blob
    ///
    pub fn move_blob(&mut self, blob_id: BlobId, new_pos: (f64, f64)) {
        if let Some(blob) = self.blobs.get_mut(&blob_id) {
            blob.pos = new_pos;
        }
    }

    ///
    /// Updates the canvas size that the BlobLand will be rendered over
    ///
    pub fn set_canvas_size(&mut self, size: (f64, f64)) {
        todo!()
    }
}

impl SampledContour for BlobLand {
    ///
    /// The size of this contour
    ///
    fn contour_size(&self) -> ContourSize {
        todo!()
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
        todo!()
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
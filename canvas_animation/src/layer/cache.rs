use flo_curves::*;

///
/// Cached values for an animation layer
///
pub struct AnimationLayerCache {
    /// Bounding boxes for all of the paths in the drawing with their index, ordered by their minimum x-coordinate
    bounding_boxes: Option<Vec<(usize, (Coord2, Coord2))>>
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
}

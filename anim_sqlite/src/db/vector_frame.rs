use super::*;
use super::flo_store::*;
use super::flo_query::*;

use canvas::*;

use std::time::Duration;

///
/// Represents a frame calculated from a vector layer
/// 
pub struct VectorFrame {
    /// The ID of the layer that this frame is for
    layer_id: i64,

    /// The ID of the keyframe that this frame is for
    keyframe_id: i64,

    /// The time of the keyframe
    keyframe_time: Duration,

    /// Time from the start of the keyframe that this frame is at
    keyframe_offset: Duration
}

impl VectorFrame {
    ///
    /// Creates a vector frame by querying the file for the frame at the specified time
    /// 
    pub fn frame_at_time<TFile: FloFile>(db: &mut TFile, layer_id: i64, when: Duration) -> Result<VectorFrame> {
        // Fetch the keyframe times
        let (keyframe_id, keyframe_time)    = db.query_nearest_key_frame(layer_id, when)?;
        let keyframe_offset                 = when - keyframe_time;

        // TODO: read the elements for this layer

        // Can create the frame now
        Ok(VectorFrame {
            layer_id:           layer_id,
            keyframe_id:        keyframe_id,
            keyframe_time:      keyframe_time,
            keyframe_offset:    keyframe_offset
        })
    }
}

impl Frame for VectorFrame {
    ///
    /// Time index of this frame
    /// 
    fn time_index(&self) -> Duration {
        self.keyframe_time + self.keyframe_offset
    }

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut GraphicsPrimitives) {
        unimplemented!()
    }
}

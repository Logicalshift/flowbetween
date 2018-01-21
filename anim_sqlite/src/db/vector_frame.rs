use super::*;
use super::flo_store::*;
use super::flo_query::*;

use canvas::*;

use std::time::Duration;

///
/// Represents a frame calculated from a vector layer
/// 
pub struct VectorFrame {

}

impl VectorFrame {
    ///
    /// Creates a vector frame by querying the file for the frame at the specified time
    /// 
    pub fn frame_at_time<TFile: FloFile>(core: &mut TFile, when: Duration) -> Result<VectorFrame> {
        unimplemented!()
    }
}

impl Frame for VectorFrame {
    ///
    /// Time index of this frame
    /// 
    fn time_index(&self) -> Duration {
        unimplemented!()
    }

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut GraphicsPrimitives) {
        unimplemented!()
    }
}

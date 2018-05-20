use super::animation_core::*;
use super::super::traits::*;

use futures::*;

use std::sync::*;

///
/// Sink for performing in-memory animation edits
/// 
pub struct AnimationSink {
    /// The animation core that this will edit
    core: Arc<Mutex<AnimationCore>>
}

impl Sink for AnimationSink {
    type SinkItem   = Vec<AnimationEdit>;
    type SinkError  = ();

    fn start_send(&mut self, item: Vec<AnimationEdit>) -> StartSend<Vec<AnimationEdit>, ()> {
        // Perform the edit directly
        self.edit(item);

        // Edit performed
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        // The in-memory sink performs all edits immediately, so is ever-ready
        Ok(Async::Ready(()))
    }
}

impl AnimationSink {
    ///
    /// Creates a new animation sink
    /// 
    pub fn new(core: Arc<Mutex<AnimationCore>>) -> AnimationSink {
        AnimationSink {
            core: core
        }
    }

    ///
    /// Performs a series of edits on this sink
    /// 
    pub fn edit(&self, edit: Vec<AnimationEdit>) {
        // Send the edits to the core
        let mut core = self.core.lock().unwrap();

        edit.into_iter()
            .for_each(|edit| core.edit(edit));
    }
}
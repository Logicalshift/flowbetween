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
    /// Assigns element IDs to a set of edits
    /// 
    fn assign_ids(&self, edits: Vec<AnimationEdit>) -> Vec<AnimationEdit> {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;
        use self::PaintEdit::*;

        let core = self.core.lock().unwrap();

        // Assign IDs to any element edits
        let mut next_element_id = core.edit_log.len() as i64;
        let mut result          = vec![];

        // Convenience function to assign the next ID
        let mut assign_id       = move || { 
            let element_id = next_element_id;
            next_element_id += 1;
            element_id
        };

        for elem in edits.into_iter() {
            let with_id = match elem {
                Layer(layer_id, Paint(when, BrushProperties(ElementId::Unassigned, props))) =>
                    Layer(layer_id, Paint(when, BrushProperties(ElementId::Assigned(assign_id()), props))),

                Layer(layer_id, Paint(when, SelectBrush(ElementId::Unassigned, defn, drawing_style))) =>
                    Layer(layer_id, Paint(when, SelectBrush(ElementId::Assigned(assign_id()), defn, drawing_style))),

                Layer(layer_id, Paint(when, BrushStroke(ElementId::Unassigned, points))) =>
                    Layer(layer_id, Paint(when, BrushStroke(ElementId::Assigned(assign_id()), points))),

                other => other
            };

            result.push(with_id);
        }

        result
    }

    ///
    /// Performs a series of edits on this sink
    /// 
    pub fn edit(&self, edits: Vec<AnimationEdit>) {
        let edits       = self.assign_ids(edits);
        let mut core    = self.core.lock().unwrap();

        // Send the edits to the core
        edits.iter()
            .for_each(|edit| core.edit(edit));

        // Store them in the edit log
        core.edit_log.extend(edits.into_iter());
    }
}
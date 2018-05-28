use super::animation_core::*;
use super::animation_sink::*;
use super::super::traits::*;

use futures::*;
use futures::stream;

use std::sync::*;
use std::collections::*;
use std::time::Duration;
use std::ops::{Range, Deref};
use std::marker::PhantomData;

///
/// Represents an animation that's stored entirely in memory 
///
pub struct InMemoryAnimation {
    /// The core contains the actual animation data
    core: Arc<Mutex<AnimationCore>>,
}

impl InMemoryAnimation {
    ///
    /// Creates a new animation
    /// 
    pub fn new() -> InMemoryAnimation {
        // Create the core (30fps by default)
        let core = AnimationCore {
            edit_log:           vec![],
            size:               (1980.0, 1080.0),
            next_element_id:    0,
            vector_layers:      HashMap::new(),
            motions:            vec![]
        };

        // Create the final animation
        InMemoryAnimation { 
            core:   Arc::new(Mutex::new(core)),
        }
    }

    ///
    /// Convenience method that performs some edits on this animation
    /// 
    pub fn perform_edits(&self, edits: Vec<AnimationEdit>) {
        let mut editor = self.edit();

        editor.start_send(edits).unwrap();
    }
}

///
/// Creates a reference to a layer within the animation core
/// 
/// Used to ensure that we retain the lock on the core while a layer editor is
/// open.
/// 
/// Rust won't infer that the target lifetime is 'a without the phantomdata
/// (or let us specify it in the impl)
/// 
struct CoreLayerRef<'a, CoreRef: 'a>(CoreRef, u64, PhantomData<&'a CoreRef>);

impl<'a, CoreRef: Deref<Target=AnimationCore>> Deref for CoreLayerRef<'a, CoreRef> {
    type Target = Layer+'a;

    fn deref(&self) -> &Self::Target {
        &*self.0.vector_layers.get(&self.1).unwrap()
    }
}

impl Animation for InMemoryAnimation {
    fn size(&self) -> (f64, f64) {
        self.core.lock().unwrap().size
    }

    fn duration(&self) -> Duration {
        Duration::from_millis(1000 * 120)
    }

    fn frame_length(&self) -> Duration {
        Duration::new(0, 1_000_000_000 / 30)
    }

    fn get_layer_ids(&self) -> Vec<u64> {
        self.core.lock().unwrap()
            .vector_layers.keys().cloned().collect()
    }

    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Box<'a+Deref<Target='a+Layer>>> {
        let core = self.core.lock().unwrap();

        if core.vector_layers.contains_key(&layer_id) {
            Some(Box::new(CoreLayerRef(core, layer_id, PhantomData)))
        } else {
            None
        }
    }

    fn get_num_edits(&self) -> usize {
        self.core.lock().unwrap().edit_log.len()
    }

    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<'a+Stream<Item=AnimationEdit, Error=()>> {
        let core        = self.core.lock().unwrap();
        let log_items   = range.into_iter().map(move |index| core.edit_log[index].clone());
        let log_items   = stream::iter_ok(log_items);

        Box::new(log_items)
    }

    fn get_motion_ids(&self, when: Range<Duration>) -> Box<Stream<Item=ElementId, Error=()>> {
        let core                = self.core.lock().unwrap();
        let when_millis         = (to_millis(when.start) as f32)..(to_millis(when.end) as f32);

        // Collect all the motions in this range into a vec (would be more efficient to store in a btree or something maybe)
        let motion_ids: Vec<_>  = core.motions.iter()
            .filter(|(_element_id, motion)| {
                let motion_time = motion.range_millis();

                !(motion_time.start > when_millis.end) && !(motion_time.end < when_millis.start)
            })
            .map(|(element_id, _motion)| *element_id)
            .collect();
        
        // Return this vec as a stream
        Box::new(stream::iter_ok(motion_ids.into_iter()))
    }

    fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
        // Flat search for a motion with this ID
        let core    = self.core.lock().unwrap();
        let motion  = core.motions.iter()
            .filter(|(element_id, _motion)| element_id == &motion_id)
            .nth(0)
            .map(|(_element_id, motion)| motion.clone());
        
        motion
    }
}

impl EditableAnimation for InMemoryAnimation {
    fn edit(&self) -> Box<Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>+Send> {
        Box::new(AnimationSink::new(Arc::clone(&self.core)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use futures::executor;

    #[test]
    fn can_set_size() {
        let animation = InMemoryAnimation::new();

        assert!(animation.size() == (1980.0, 1080.0));

        animation.perform_edits(vec![
            AnimationEdit::SetSize(800.0, 600.0)
        ]);

        assert!(animation.size() == (800.0, 600.0));
    }

    #[test]
    fn can_add_layer() {
        let animation = InMemoryAnimation::new();

        assert!(animation.get_layer_ids().len() == 0);

        animation.perform_edits(vec![
            AnimationEdit::AddNewLayer(0)
        ]);

        assert!(animation.get_layer_ids().len() == 1);
    }

    #[test]
    fn can_remove_layer() {
        let animation = InMemoryAnimation::new();

        let keep1       = 0;
        let keep2       = 1;
        let to_remove   = 2;
        let keep3       = 3;

        animation.perform_edits(vec![
            AnimationEdit::AddNewLayer(keep1),
            AnimationEdit::AddNewLayer(keep2),
            AnimationEdit::AddNewLayer(to_remove),
            AnimationEdit::AddNewLayer(keep3),
        ]);

        let mut ids = animation.get_layer_ids();
        ids.sort();
        assert!(ids == vec![keep1, keep2, to_remove, keep3]);

        animation.perform_edits(vec![
            AnimationEdit::RemoveLayer(to_remove)
        ]);

        let mut ids = animation.get_layer_ids();
        ids.sort();
        assert!(ids == vec![keep1, keep2, keep3]);
    }

    #[test]
    fn will_assign_element_ids() {
        let animation = InMemoryAnimation::new();

        // Perform some edits on the animation with an unassigned element ID
        animation.perform_edits(vec![
            AnimationEdit::AddNewLayer(0),
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
            AnimationEdit::Layer(0, LayerEdit::Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ]))))
        ]);

        // Element ID should be assigned if we read the log back
        let edit_log    = animation.read_edit_log(2..3);
        let paint_edit  = executor::spawn(edit_log.collect()).wait_future().unwrap();

        // Should be able to find the paint edit here
        assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, _)) => true, _ => false });

        // Element ID should be assigned
        assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, PaintEdit::BrushStroke(ElementId::Assigned(_), _))) => true, _ => false });
    }

    #[test]
    fn can_draw_brush_stroke() {
        let animation = InMemoryAnimation::new();

        animation.perform_edits(vec![
            AnimationEdit::AddNewLayer(0),
        ]);

        assert!(animation.get_layer_ids().len() == 1);

        // Add a keyframe
        animation.perform_edits(vec![
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        ]);

        // Draw a brush stroke
        {
            let mut layer_edit = animation.edit();

            layer_edit.start_send(vec![
                AnimationEdit::Layer(0,
                    LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                        RawPoint::from((10.0, 10.0)),
                        RawPoint::from((20.0, 5.0))
                    ])))
                )
            ]).unwrap();
        }
    }
}

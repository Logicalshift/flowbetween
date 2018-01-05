use super::edit_log::*;
use super::vector_layer::*;
use super::super::traits::*;

use std::sync::*;
use std::ops::Range;
use std::time::Duration;
use std::collections::*;

///
/// Core values associated with an animation
/// 
struct AnimationCore {
    /// A weak reference back to this core (used when we need to pass it around for editing purposes)
    self_reference: Weak<RwLock<AnimationCore>>,

    /// The edit log for this animation
    edit_log: InMemoryEditLog<AnimationEdit>,

    /// The size of the animation canvas
    size: (f64, f64),

    /// The duration of a frame in the animation
    frame_duration: Duration,

    /// The layers in this animation, as an object and as a vector layer (we need to return references to the layer object and rust can't downgrade for us)
    layers: Vec<(Arc<Layer>, Arc<VectorLayer>)>,
}

///
/// Represents an animation that's stored entirely in memory 
///
pub struct InMemoryAnimation {
    /// The core contains the actual animation data
    core: Arc<RwLock<AnimationCore>>
}

impl InMemoryAnimation {
    pub fn new() -> InMemoryAnimation {
        // Create the core (30fps by default)
        let core = AnimationCore {
            self_reference:     Weak::default(),
            edit_log:           InMemoryEditLog::new(),
            size:               (1980.0, 1080.0),
            frame_duration:     Duration::from_millis(1000/30),
            layers:             vec![]
        };

        // Core needs a self-reference so it can supply itself as the edit log for layers
        let core            = Arc::new(RwLock::new(core));
        let self_reference  = Arc::downgrade(&core);

        core.write().unwrap().self_reference = self_reference;

        // Create the final animation
        InMemoryAnimation { core: core }
    }
}

impl Animation for InMemoryAnimation { 
    fn size(&self) -> (f64, f64) {
        (*self.core).read().unwrap().size
    }
}

impl Editable<AnimationLayers+'static> for InMemoryAnimation {
    fn edit(&self) -> Option<Editor<AnimationLayers+'static>> { 
        let core: &RwLock<AnimationLayers>  = &*self.core;

        Some(Editor::new(core.write().unwrap()))
    }

    fn read(&self) -> Option<Reader<AnimationLayers+'static>> { 
        let core: &RwLock<AnimationLayers>  = &*self.core;

        Some(Reader::new(core.read().unwrap()))
    }
}

impl Editable<EditLog<AnimationEdit>> for InMemoryAnimation {
    fn edit(&self) -> Option<Editor<EditLog<AnimationEdit>+'static>> {
        None
    }

    fn read(&self) -> Option<Reader<EditLog<AnimationEdit>+'static>> { 
        let core: &RwLock<EditLog<AnimationEdit>>  = &*self.core;

        Some(Reader::new(core.read().unwrap()))
    }
}

impl Editable<MutableEditLog<AnimationEdit>> for InMemoryAnimation {
    fn edit(&self) -> Option<Editor<MutableEditLog<AnimationEdit>+'static>> { 
        let core: &RwLock<MutableEditLog<AnimationEdit>>  = &*self.core;

        Some(Editor::new(core.write().unwrap()))
    }

    fn read(&self) -> Option<Reader<MutableEditLog<AnimationEdit>+'static>> { 
        None
    }
}

impl AnimationCore {
    fn remove_layer(&mut self, layer_id: u64) {
        // Find the index of the layer with this ID
        let remove_index = {
            let mut remove_index = None;

            for index in 0..self.layers.len() {
                if self.layers[index].1.id() == layer_id {
                    remove_index = Some(index);
                }
            }
            remove_index
        };

        // Remove this layer
        if let Some(remove_index) = remove_index {
            self.layers.remove(remove_index);
        }
    }

    fn add_new_layer(&mut self, layer_id: u64) {
        // TODO: do nothing if the layer does not exist

        // We need a self-reference to act as the edit log
        if let Some(edit_log) = self.self_reference.upgrade() {
            let edit_log: Arc<RwLock<MutableEditLog<AnimationEdit>>> = edit_log.clone();

            // Generate the layer
            let new_layer = Arc::new(VectorLayer::new(layer_id, &edit_log));
            self.layers.push((new_layer.clone(), new_layer));
        }
    }

    fn set_size(&mut self, new_size: (f64, f64)) {
        self.size = new_size;
    }
}

impl EditLog<AnimationEdit> for AnimationCore {
    fn length(&self) -> usize {
        self.edit_log.length()
    }

    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<AnimationEdit> {
        self.edit_log.read(indices)
    }

    fn pending(&self) -> Vec<AnimationEdit> {
        self.edit_log.pending()
    }
}

impl MutableEditLog<AnimationEdit> for AnimationCore {
    fn set_pending(&mut self, edits: &[AnimationEdit]) {
        // TODO: the layers probably want to know about pending stuff that affects them
        self.edit_log.set_pending(edits)
    }

    fn cancel_pending(&mut self) {
        self.edit_log.cancel_pending()
    }

    fn commit_pending(&mut self) -> Range<usize> {
        // Get the items that are currently pending
        let to_process = self.edit_log.pending();

        // Commit the pending items to the log
        let commit_range = self.edit_log.commit_pending();

        let mut layer_edits: HashMap<u64, Vec<LayerEdit>> = HashMap::new();

        // Perform the animation edits
        for action in to_process {
            use AnimationEdit::*;
            
            match action {
                DefineBrush(_, _)       => { unimplemented!(); },

                Layer(layer_id, edit)   => { 
                    let edits = layer_edits.entry(layer_id).or_insert(vec![]);
                    edits.push(edit); 
                },

                SetSize(x, y)           => { self.set_size((x, y)); },
                AddNewLayer(id)         => { self.add_new_layer(id); },
                RemoveLayer(id)         => { self.remove_layer(id); }
            }
        }

        // Finish the layer edits independently
        for (layer_id, edits) in layer_edits {
            if let Some(layer) = self.layers.iter().find(|layer| layer.1.id() == layer_id) {
                layer.1.apply_new_edits(&edits);
            }
        }

        commit_range
    }
}

impl AnimationLayers for AnimationCore {
    fn layers<'a>(&'a self) -> Box<'a+Iterator<Item = &'a Arc<Layer>>> {
        Box::new(self.layers.iter().map(|&(ref layer, _)| layer))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;

    #[test]
    fn can_add_layer() {
        let animation = InMemoryAnimation::new();

        {
            let layers = open_read::<AnimationLayers>(&animation).unwrap();
            assert!(layers.layers().count() == 0);
        }

        animation.perform_edits(vec![
            AnimationEdit::AddNewLayer(0)
        ]);

        {
            let layers = open_read::<AnimationLayers>(&animation).unwrap();
            assert!(layers.layers().count() == 1);
        }
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

        {
            let layers = open_read::<AnimationLayers>(&animation).unwrap();
            let ids: Vec<u64> = layers.layers().map(|layer| layer.id()).collect();
            assert!(ids == vec![keep1, keep2, to_remove, keep3]);
        }

        animation.perform_edits(vec![
            AnimationEdit::RemoveLayer(to_remove)
        ]);

        {
            let layers = open_read::<AnimationLayers>(&animation).unwrap();
            let ids: Vec<u64> = layers.layers().map(|layer| layer.id()).collect();
            assert!(ids == vec![keep1, keep2, keep3]);
        }
    }

    #[test]
    fn can_draw_brush_stroke() {
        let animation = InMemoryAnimation::new();

        animation.perform_edits(vec![
            AnimationEdit::AddNewLayer(0),
        ]);

        {
            let layers = open_edit::<AnimationLayers>(&animation).unwrap();
            assert!(layers.layers().count() == 1);
        }

        // Add a keyframe
        animation.perform_edits(vec![
            AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        ]);

        {
            let layers = open_edit::<AnimationLayers>(&animation).unwrap();
            let layer = layers.layers().nth(0).unwrap();

            // Draw a brush stroke
            let mut brush: Editor<PaintLayer> = layer.edit().unwrap();

            brush.start_brush_stroke(Duration::from_millis(442), BrushPoint::from((0.0, 0.0)));
            brush.continue_brush_stroke(BrushPoint::from((10.0, 10.0)));
            brush.continue_brush_stroke(BrushPoint::from((20.0, 5.0)));
            brush.finish_brush_stroke();

            mem::drop(brush);
        }
    }
}

use super::edit_log::*;
use super::pending_log::*;
use super::vector_layer::*;
use super::super::traits::*;
use super::super::editor::*;

use std::sync::*;
use std::collections::*;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Range};

///
/// Core values associated with an animation
/// 
struct AnimationCore {
    /// The edit log for this animation
    edit_log: InMemoryEditLog<AnimationEdit>,

    /// The size of the animation canvas
    size: (f64, f64),

    /// The layers in this animation, as an object and as a vector layer (we need to return references to the layer object and rust can't downgrade for us)
    layers: HashMap<u64, Box<Layer>>,
}

///
/// Represents an animation that's stored entirely in memory 
///
pub struct InMemoryAnimation {
    /// The core contains the actual animation data
    core: Arc<RwLock<AnimationCore>>,
}

impl InMemoryAnimation {
    ///
    /// Creates a new animation
    /// 
    pub fn new() -> InMemoryAnimation {
        // Create the core (30fps by default)
        let core = AnimationCore {
            edit_log:           InMemoryEditLog::new(),
            size:               (1980.0, 1080.0),
            layers:             HashMap::new()
        };

        // Create the final animation
        InMemoryAnimation { 
            core:   Arc::new(RwLock::new(core)),
        }
    }
}

///
/// Creates a reference to a layer within the animation core
/// 
/// Rust won't infer that the target lifetime is 'a without the phantomdata
/// (or let us specify it in the impl)
/// 
struct CoreLayerRef<'a, CoreRef: 'a>(CoreRef, u64, PhantomData<&'a CoreRef>);

impl<'a, CoreRef: Deref<Target=AnimationCore>> Deref for CoreLayerRef<'a, CoreRef> {
    type Target = Layer+'a;

    fn deref(&self) -> &Self::Target {
        &**self.0.layers.get(&self.1).unwrap()
    }
}

impl<'a, CoreRef: Deref<Target=AnimationCore>+DerefMut> DerefMut for CoreLayerRef<'a, CoreRef> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut **self.0.layers.get_mut(&self.1).unwrap()
    }
}

impl Animation for InMemoryAnimation {
    fn size(&self) -> (f64, f64) {
        (*self.core).read().unwrap().size
    }

    fn get_layer_ids(&self) -> Vec<u64> {
        (*self.core).read().unwrap()
            .layers.keys().cloned().collect()
    }

    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Reader<'a, Layer>> {
        let core = (*self.core).read().unwrap();

        if core.layers.contains_key(&layer_id) {
            let layer_ref   = CoreLayerRef(core, layer_id, PhantomData);
            let reader      = Reader::new(layer_ref);

            Some(reader)
        } else {
            None
        }
    }

    fn get_log<'a>(&'a self) -> Reader<'a, EditLog<AnimationEdit>> {
        let core: &RwLock<EditLog<AnimationEdit>> = &*self.core;

        Reader::new(core.read().unwrap())
    }

    fn edit<'a>(&'a self) -> Editor<'a, PendingEditLog<AnimationEdit>> {
        let core = self.core.clone();

        // Create an edit log that will commit to this object's log
        let edit_log = InMemoryPendingLog::new(move |edits| core.write().unwrap().commit_edits(edits));

        // Turn it into an editor
        let edit_log: Box<'a+PendingEditLog<AnimationEdit>> = Box::new(edit_log);
        Editor::new(edit_log)
    }

    fn edit_layer<'a>(&'a self, layer_id: u64) -> Editor<'a, PendingEditLog<LayerEdit>> {
        let core = self.core.clone();

        // Create an edit log that will commit to this object's log
        let edit_log = InMemoryPendingLog::new(move |edits| {
            let edits = edits.into_iter()
                .map(|edit| AnimationEdit::Layer(layer_id, edit));

            core.write().unwrap().commit_edits(edits)
        });

        // Turn it into an editor
        let edit_log: Box<'a+PendingEditLog<LayerEdit>> = Box::new(edit_log);
        Editor::new(edit_log)
    }
}

impl AnimationCore {
    ///
    /// Commits a set of edits to this animation
    /// 
    fn commit_edits<I: IntoIterator<Item=AnimationEdit>>(&mut self, edits: I) -> Range<usize> {
        // The animation editor is what actually applies these edits to this object
        let editor = AnimationEditor::new();

        // Collect the edits into a vec so we can inspect them multiple times
        let edits: Vec<AnimationEdit> = edits.into_iter().collect();

        // Process the edits in the core
        editor.perform(self, edits.iter().cloned());

        // Commit to the main log
        self.edit_log.commit_edits(edits)
    }
}

impl MutableAnimation for AnimationCore {
    ///
    /// Sets the canvas size of this animation
    ///
    fn set_size(&mut self, size: (f64, f64)) {
        self.size = size;
    }

    ///
    /// Creates a new layer with a particular ID
    /// 
    /// Has no effect if the layer ID is already in use
    /// 
    fn add_layer(&mut self, new_layer_id: u64) {
        self.layers.entry(new_layer_id)
            .or_insert_with(|| {
                let layer = InMemoryVectorLayer::new(new_layer_id);

                Box::new(layer)
            });
    }

    ///
    /// Removes the layer with the specified ID
    /// 
    fn remove_layer(&mut self, old_layer_id: u64) {
        self.layers.remove(&old_layer_id);
    }

    ///
    /// Opens a particular layer for editing
    /// 
    fn edit_layer<'a>(&'a mut self, layer_id: u64) -> Option<Editor<'a, Layer>> {
        unimplemented!()
    }
}

impl EditLog<AnimationEdit> for AnimationCore {
    fn length(&self) -> usize {
        self.edit_log.length()
    }

    fn read(&self, indices: &mut Iterator<Item=usize>) -> Vec<AnimationEdit> {
        self.edit_log.read(indices)
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

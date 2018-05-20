use super::edit_log::*;
use super::pending_log::*;
use super::vector_layer::*;
use super::super::traits::*;
use super::super::editor::*;

use std::sync::*;
use std::collections::*;
use std::time::Duration;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

///
/// Core values associated with an animation
/// 
struct AnimationCore {
    /// The edit log for this animation
    edit_log: InMemoryEditLog<AnimationEdit>,

    /// The next element ID to assign
    next_element_id: i64,

    /// The size of the animation canvas
    size: (f64, f64),

    /// The vector layers in this animation
    vector_layers: HashMap<u64, InMemoryVectorLayer>,
}

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
            edit_log:           InMemoryEditLog::new(),
            size:               (1980.0, 1080.0),
            next_element_id:    0,
            vector_layers:      HashMap::new()
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
        editor.set_pending(&edits);
        editor.commit_pending();
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

impl<'a, CoreRef: Deref<Target=AnimationCore>+DerefMut> DerefMut for CoreLayerRef<'a, CoreRef> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0.vector_layers.get_mut(&self.1).unwrap()
    }
}

impl Animation for InMemoryAnimation {
    fn size(&self) -> (f64, f64) {
        (*self.core).lock().unwrap().size
    }

    fn duration(&self) -> Duration {
        Duration::from_millis(1000 * 120)
    }

    fn frame_length(&self) -> Duration {
        Duration::new(0, 1_000_000_000 / 30)
    }

    fn get_layer_ids(&self) -> Vec<u64> {
        (*self.core).lock().unwrap()
            .vector_layers.keys().cloned().collect()
    }

    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Reader<'a, Layer>> {
        let core = (*self.core).lock().unwrap();

        if core.vector_layers.contains_key(&layer_id) {
            let layer_ref   = CoreLayerRef(core, layer_id, PhantomData);
            let reader      = Reader::new(layer_ref);

            Some(reader)
        } else {
            None
        }
    }

    fn get_log<'a>(&'a self) -> Reader<'a, EditLog<AnimationEdit>> {
        let core: &Mutex<EditLog<AnimationEdit>> = &*self.core;

        Reader::new(core.lock().unwrap())
    }
}

impl AnimationCore {
    ///
    /// Commits a set of edits to this animation
    /// 
    fn commit_edits<I: IntoIterator<Item=AnimationEdit>>(&mut self, edits: I) {
        // The animation editor is what actually applies these edits to this object
        let editor = AnimationEditor::new();

        // Collect the edits and assign element IDs as we go
        let mut assigned_edits = vec![];
        for edit in edits {
            assigned_edits.push(edit.assign_element_id(|| {
                let id = self.next_element_id;
                self.next_element_id += 1;
                id
            }));         
        }

        // Process the edits in the core
        editor.perform(self, assigned_edits.iter().cloned());

        // Commit to the main log
        self.edit_log.commit_edits(assigned_edits);
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
        self.vector_layers.entry(new_layer_id)
            .or_insert_with(|| InMemoryVectorLayer::new(new_layer_id));
    }

    ///
    /// Removes the layer with the specified ID
    /// 
    fn remove_layer(&mut self, old_layer_id: u64) {
        self.vector_layers.remove(&old_layer_id);
    }

    ///
    /// Opens a particular layer for editing
    /// 
    fn edit_layer<'a>(&'a mut self, layer_id: u64) -> Option<Editor<'a, Layer>> {
        if self.vector_layers.contains_key(&layer_id) {
            let layer_ref   = CoreLayerRef(self, layer_id, PhantomData);
            let reader      = Editor::new(layer_ref);

            Some(reader)
        } else {
            None
        }
    }

    ///
    /// Performs an edit on an element contained within this animation
    /// 
    fn edit_element(&mut self, element_id: ElementId, when: Duration, edit: ElementEdit) {
        // Forward this edit to all of the layers (the one that owns the element will carry it out)
        for (_id, layer) in self.vector_layers.iter_mut() {
            layer.edit_element(element_id, when, &edit);
        }
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
    use std::time::Duration;

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
        let edit_log = animation.get_log();

        let paint_edit = edit_log.read(&mut (2..3));

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
            let mut layer_edit = animation.edit_layer(0);

            layer_edit.set_pending(&vec![
                LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))
            ]);
            layer_edit.commit_pending();
        }
    }
}

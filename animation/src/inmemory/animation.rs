use super::vector_layer::*;
use super::super::traits::*;

use std::sync::*;
use std::time::Duration;

///
/// Core values associated with an animation
/// 
struct AnimationCore {
    /// The size of the animation canvas
    size: (f64, f64),

    /// The duration of a frame in the animation
    frame_duration: Duration,

    /// The layers in this animation
    layers: Vec<Arc<Layer>>,

    next_layer_id: u64
}

///
/// Represents an animation that's stored entirely in memory 
///
pub struct InMemoryAnimation {
    /// The core contains the actual animation data
    core: RwLock<AnimationCore>
}

impl InMemoryAnimation {
    pub fn new() -> InMemoryAnimation {
        // Create the core (30fps by default)
        let core = AnimationCore { 
            size:           (1980.0, 1080.0),
            frame_duration: Duration::from_millis(1000/30),
            layers:         vec![],
            next_layer_id:  0
        };

        // Create the final animation
        InMemoryAnimation { core: RwLock::new(core) }
    }
}

impl Animation for InMemoryAnimation { }

impl Editable<AnimationSize+'static> for InMemoryAnimation {
    fn edit(&self) -> Option<Editor<AnimationSize+'static>> {
        // (Need the explicit typing here as rust can't figure it out implicitly)
        let core: &RwLock<AnimationSize>    = &self.core;
        let core                            = core.write().unwrap();

        Some(Editor::new(core))
    }

    fn read(&self) -> Option<Reader<AnimationSize+'static>> {
        let core: &RwLock<AnimationSize>    = &self.core;
        let core                            = core.read().unwrap();

        Some(Reader::new(core))
    }
}

impl Editable<AnimationLayers+'static> for InMemoryAnimation {
    fn edit(&self) -> Option<Editor<AnimationLayers+'static>> { 
        let core: &RwLock<AnimationLayers>  = &self.core;

        Some(Editor::new(core.write().unwrap()))
    }

    fn read(&self) -> Option<Reader<AnimationLayers+'static>> { 
        let core: &RwLock<AnimationLayers>  = &self.core;

        Some(Reader::new(core.read().unwrap()))
    }
}

impl AnimationSize for AnimationCore {
    fn size(&self) -> (f64, f64) { self.size }

    fn set_size(&mut self, new_size: (f64, f64)) {
        self.size = new_size;
    }
}

impl AnimationLayers for AnimationCore {
    fn layers<'a>(&'a self) -> Box<'a+Iterator<Item = &'a Arc<Layer>>> {
        Box::new(self.layers.iter())
    }

    fn remove_layer(&mut self, layer_id: u64) {
        // Find the index of the layer with this ID
        let remove_index = {
            let mut remove_index = None;

            for index in 0..self.layers.len() {
                if self.layers[index].id() == layer_id {
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

    fn add_new_layer<'a>(&'a mut self) -> &'a Layer {
        // Pick an ID for this layer
        let layer_id = self.next_layer_id;
        self.next_layer_id += 1;

        // Generate the layer
        let new_layer = Arc::new(VectorLayer::new(layer_id));
        self.layers.push(new_layer);

        // Result is a reference to the layer
        &**self.layers.last().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;

    #[test]
    fn can_add_layer() {
        let animation = InMemoryAnimation::new();
        let mut layers = open_edit::<AnimationLayers>(&animation).unwrap();

        assert!(layers.layers().count() == 0);

        layers.add_new_layer();
        assert!(layers.layers().count() == 1);
    }

    #[test]
    fn can_remove_layer() {
        let animation = InMemoryAnimation::new();
        let mut layers = open_edit::<AnimationLayers>(&animation).unwrap();

        assert!(layers.layers().count() == 0);

        let keep1       = layers.add_new_layer().id();
        let keep2       = layers.add_new_layer().id();
        let to_remove   = layers.add_new_layer().id();
        let keep3       = layers.add_new_layer().id();

        let ids: Vec<u64> = layers.layers().map(|layer| layer.id()).collect();
        assert!(ids == vec![keep1, keep2, to_remove, keep3]);

        layers.remove_layer(to_remove);

        let ids: Vec<u64> = layers.layers().map(|layer| layer.id()).collect();
        assert!(ids == vec![keep1, keep2, keep3]);
    }

    #[test]
    fn can_draw_brush_stroke() {
        let animation = InMemoryAnimation::new();
        let mut layers = open_edit::<AnimationLayers>(&animation).unwrap();

        assert!(layers.layers().count() == 0);

        let layer = layers.add_new_layer();

        // Add a keyframe
        let mut keyframes: Editor<KeyFrameLayer> = layer.edit().unwrap();

        keyframes.add_key_frame(Duration::from_millis(0));

        mem::drop(keyframes);

        // Draw a brush stroke
        let mut brush: Editor<PaintLayer> = layer.edit().unwrap();

        brush.start_brush_stroke(Duration::from_millis(442), BrushPoint::from((0.0, 0.0)));
        brush.continue_brush_stroke(BrushPoint::from((10.0, 10.0)));
        brush.continue_brush_stroke(BrushPoint::from((20.0, 5.0)));
        brush.finish_brush_stroke();

        mem::drop(brush);
    }
}

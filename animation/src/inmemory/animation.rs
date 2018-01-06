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
            edit_log:           InMemoryEditLog::new(),
            size:               (1980.0, 1080.0),
            frame_duration:     Duration::from_millis(1000/30),
            layers:             vec![]
        };

        // Create the final animation
        InMemoryAnimation { core: Arc::new(RwLock::new(core)) }
    }
}

impl AnimationCore {
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

use crate::*;
use crate::storage::*;
use crate::undo::*;

use flo_stream::*;
use flo_canvas::*;

use futures::prelude::*;
use futures::executor;

use std::sync::*;
use std::time::{Duration};

///
/// Creates path components for a circular path
///
fn circle_path(pos: (f64, f64), radius: f64) -> Arc<Vec<PathComponent>> {
    let mut drawing = vec![];

    drawing.new_path();
    drawing.circle(pos.0 as _, pos.1 as _, radius as _);

    let path        = Path::from_drawing(drawing);

    Arc::new(path.elements().collect())
}

#[test]
fn create_element() {
    executor::block_on(async {
        use AnimationEdit::*;
        use LayerEdit::*;

        // Create the animation
        let in_memory_store = InMemoryStorage::new();
        let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());
        let animation       = UndoableAnimation::new(animation);

        // Setup a layer
        animation.edit().publish(Arc::new(vec![
            AddNewLayer(0),
            Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
            Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(20000))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        // Create a single element
        animation.edit().publish(Arc::new(vec![
                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Undo(UndoEdit::FinishAction),
            ])).await;

        // Wait for the edits to commit
        animation.edit().when_empty().await;

        // Element should exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 1);
    });
}

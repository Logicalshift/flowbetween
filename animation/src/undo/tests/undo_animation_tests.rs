use crate::*;
use crate::storage::*;
use crate::undo::*;

use flo_stream::*;
use flo_canvas::*;

use futures::prelude::*;
use futures::executor;
use futures::future::{select, Either};
use futures_timer::{Delay};

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

#[test]
fn undo_create_element() {
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
            Undo(UndoEdit::FinishAction),
        ])).await;

        animation.edit().publish(Arc::new(vec![
            Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        // Wait for the edits to commit
        animation.edit().when_empty().await;

        // Element should exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 1);

        // Undo the create action
        let timeout     = Delay::new(Duration::from_secs(10));
        let undo_result = match select(animation.undo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", undo_result);
        assert!(undo_result.is_ok());

        // Element should no longer exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 0);
    });
}

#[test]
fn double_undo() {
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
            Undo(UndoEdit::FinishAction),
        ])).await;

        animation.edit().publish(Arc::new(vec![
            Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        animation.edit().publish(Arc::new(vec![
            Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        // Wait for the edits to commit
        animation.edit().when_empty().await;

        // Element should exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 2);

        // Undo the first create action
        let timeout     = Delay::new(Duration::from_secs(10));
        let undo_result = match select(animation.undo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", undo_result);
        assert!(undo_result.is_ok());

        // One element should be removed
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 1);

        // Undo the second create action
        let timeout     = Delay::new(Duration::from_secs(10));
        let undo_result = match select(animation.undo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", undo_result);
        assert!(undo_result.is_ok());

        // Both elements should no longer exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 0);
    });
}

#[test]
fn redo_create_element() {
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
            Undo(UndoEdit::FinishAction),
        ])).await;

        animation.edit().publish(Arc::new(vec![
            Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        // Wait for the edits to commit
        animation.edit().when_empty().await;

        // Element should exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 1);

        // Undo the create action
        let timeout     = Delay::new(Duration::from_secs(10));
        let undo_result = match select(animation.undo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", undo_result);
        assert!(undo_result.is_ok());

        // Element should no longer exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 0);

        // Redo the create action
        let timeout     = Delay::new(Duration::from_secs(10));
        let redo_result = match select(animation.redo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", redo_result);
        assert!(redo_result.is_ok());

        // Element should be back
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 1);
    });
}

#[test]
fn double_redo() {
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
            Undo(UndoEdit::FinishAction),
        ])).await;

        animation.edit().publish(Arc::new(vec![
            Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        animation.edit().publish(Arc::new(vec![
            Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        // Wait for the edits to commit
        animation.edit().when_empty().await;

        // Element should exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 2);
        assert!(elements[0].id() == ElementId::Assigned(0));
        assert!(elements[1].id() == ElementId::Assigned(1));

        // Undo the first create action
        let timeout     = Delay::new(Duration::from_secs(10));
        let undo_result = match select(animation.undo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", undo_result);
        assert!(undo_result.is_ok());

        // One element should be removed
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 1);

        // Undo the second create action
        let timeout     = Delay::new(Duration::from_secs(10));
        let undo_result = match select(animation.undo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", undo_result);
        assert!(undo_result.is_ok());

        // Both elements should no longer exist
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 0);

        // Redo the last two edits
        let timeout     = Delay::new(Duration::from_secs(10));
        let redo_result = match select(animation.redo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", redo_result);
        assert!(redo_result.is_ok());

        let timeout     = Delay::new(Duration::from_secs(10));
        let redo_result = match select(animation.redo().boxed(), timeout).await {
            Either::Right(_)        => { assert!(false, "Timed out"); unimplemented!() }
            Either::Left(result)    => result.0,
        };
        println!("{:?}", redo_result);
        assert!(redo_result.is_ok());

        // Both elements should be back
        let frame       = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
        let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

        assert!(elements.len() == 2);
        assert!(elements[0].id() == ElementId::Assigned(0));
        assert!(elements[1].id() == ElementId::Assigned(1));
    });
}

#[test]
fn follow_undo_log_size() {
    executor::block_on(async {
        use AnimationEdit::*;
        use LayerEdit::*;

        // Create the animation
        let in_memory_store     = InMemoryStorage::new();
        let animation           = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());
        let animation           = UndoableAnimation::new(animation);
        let mut undo_log_size   = animation.follow_undo_log_size_changes();

        let next_log_size       = undo_log_size.next().await;
        assert!(next_log_size == Some(UndoLogSize { undo_depth: 0, redo_depth: 0 }));

        // Setup a layer
        animation.edit().publish(Arc::new(vec![
            AddNewLayer(0),
            Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
            Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(20000))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        let next_log_size       = undo_log_size.next().await;
        assert!(next_log_size == Some(UndoLogSize { undo_depth: 1, redo_depth: 0 }));

        // Create a single element
        animation.edit().publish(Arc::new(vec![
            Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
            Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

            Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
            Undo(UndoEdit::FinishAction),
        ])).await;

        let next_log_size       = undo_log_size.next().await;
        assert!(next_log_size == Some(UndoLogSize { undo_depth: 2, redo_depth: 0 }));
    });
}

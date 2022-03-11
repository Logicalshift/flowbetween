use crate::*;
use crate::editor::*;
use crate::storage::*;

use super::undo_element_edits::*;

use flo_stream::*;
use flo_canvas::*;

use futures::prelude::*;
use futures::executor;
use futures::future::{select, Either};
use futures_timer::{Delay};

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Data for all of the keyframes in a layer
///
#[derive(PartialEq, Clone, Debug)]
pub struct LayerData {
    /// Data at each of the keyframes for this layer
    pub keyframes: HashMap<Duration, FrameData>
}

///
/// Reads a layer and all of its keyframes for comparison
///
pub async fn read_layer(animation: &impl Animation, layer_id: u64) -> Option<LayerData> {
    let layer               = animation.get_layer_with_id(layer_id)?;
    let keyframe_times      = layer.get_key_frames();
    let mut keyframe_data   = HashMap::new();

    // Read the frame for each keyframe (note that this does not read things like elements that are introduced later on)
    for keyframe_time in keyframe_times {
        let keyframe = read_frame(animation, layer_id, keyframe_time).await;
        keyframe_data.insert(keyframe_time, keyframe);
    }

    Some(LayerData {
        keyframes: keyframe_data
    })
}

///
/// Reads all of the layers in an animation
///
pub async fn read_all_layers(animation: &impl Animation) -> Vec<(u64, LayerData)> {
    let layer_ids   = animation.get_layer_ids();
    let mut layers  = vec![];

    for layer_id in layer_ids {
        let layer_data = read_layer(animation, layer_id).await;
        let layer_data = layer_data.expect("All layers listed in the animation should exist");
        layers.push((layer_id, layer_data));
    }

    layers
}

///
/// Tests that a layer has the same content before and and after undoing another edit
///
async fn test_layer_edit_undo(setup: Vec<AnimationEdit>, undo_test: Vec<AnimationEdit>) {
    // Create the animation
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    // Perform the setup edits
    animation.edit().publish(Arc::new(setup)).await;
    animation.edit().when_empty().await;

    // Read the initial set of layers
    let initial_layers      = read_all_layers(&animation).await;

    // The undo action appears when the edits are retired: start reading them from this point
    let timeout             = Delay::new(Duration::from_secs(10));
    let mut retired_edits   = animation.retired_edits();

    // Publish the undo test edits
    let undo_test           = Arc::new(undo_test);
    animation.edit().publish(Arc::clone(&undo_test)).await;
    animation.edit().when_empty().await;

    // The next set of edits from the retired_edits stream should be the undo edits
    let retired_edit    = match select(retired_edits.next(), timeout).await {
        Either::Right(_)    => { assert!(false, "Timed out"); unimplemented!() }
        Either::Left(edits) => edits.0.unwrap()
    };

    let committed       = retired_edit.committed_edits();
    let reverse         = retired_edit.reverse_edits();

    println!("Committed: {}", committed.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));
    println!("Reverse: {}", reverse.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

    // Sanity check: we should be able to detect the edit made by the test edits
    let during_edit     = read_all_layers(&animation).await;
    assert!(initial_layers != during_edit);

    // These edits should be equivalent (assuming the example doesn't use unassigned IDs, as the IDs will be assigned at this point)
    assert!(committed == undo_test);

    // The reverse actions should be non-empty (there are ways to create edits that have no effect, but the assumption is the tests won't do this)
    assert!(!reverse.is_empty());

    // Undo the actions
    animation.edit().publish(Arc::clone(&reverse)).await;
    animation.edit().when_empty().await;

    // Re-read the layers after performing and undoing the action
    let after_layers    = read_all_layers(&animation).await;

    // The undo action should have restored the original state
    println!();
    println!("=== INITIAL");
    println!("{:?}", initial_layers);

    println!();
    println!("=== AFTER");
    println!("{:?}", after_layers);
    assert!(initial_layers == after_layers);
}

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
fn remove_simple_layer() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
            ],
            vec![
                RemoveLayer(0)
            ]
        ).await;
    });
}

#[test]
fn remove_layer_multi_elements() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                RemoveLayer(0)
            ]
        ).await;
    });
}

#[test]
fn remove_layer_with_group() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal)),
            ],
            vec![
                RemoveLayer(0)
            ]
        ).await;
    });
}

#[test]
fn remove_layer_with_multiple_keyframes() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),
                Layer(0, AddKeyFrame(Duration::from_millis(1000))),
                Layer(0, AddKeyFrame(Duration::from_millis(2000))),
                Layer(0, AddKeyFrame(Duration::from_millis(3000))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),

                Layer(0, Path(Duration::from_millis(1000), PathEdit::SelectBrush(ElementId::Assigned(102), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(1000), PathEdit::BrushProperties(ElementId::Assigned(103), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(1000), PathEdit::CreatePath(ElementId::Assigned(3), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(1000), PathEdit::CreatePath(ElementId::Assigned(4), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(1000), PathEdit::CreatePath(ElementId::Assigned(5), circle_path((100.0, 200.0), 50.0)))),

                Layer(0, Path(Duration::from_millis(3000), PathEdit::SelectBrush(ElementId::Assigned(104), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(3000), PathEdit::BrushProperties(ElementId::Assigned(105), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(3000), PathEdit::CreatePath(ElementId::Assigned(6), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(3000), PathEdit::CreatePath(ElementId::Assigned(7), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(3000), PathEdit::CreatePath(ElementId::Assigned(8), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                RemoveLayer(0)
            ]
        ).await;
    });
}

#[test]
fn remove_layer_with_nested_group() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal)),
                Element(vec![ElementId::Assigned(3), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(4), GroupType::Normal)),
            ],
            vec![
                RemoveLayer(0)
            ]
        ).await;
    });
}

#[test]
fn remove_first_layer() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),
                AddNewLayer(1),
                Layer(1, AddKeyFrame(Duration::from_millis(0))),
                AddNewLayer(2),
                Layer(2, AddKeyFrame(Duration::from_millis(0))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),

                Layer(1, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(102), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(103), BrushProperties::new()))),

                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(3), circle_path((100.0, 100.0), 50.0)))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(4), circle_path((100.0, 150.0), 50.0)))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(5), circle_path((100.0, 200.0), 50.0)))),

                Layer(2, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(104), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(105), BrushProperties::new()))),

                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(6), circle_path((100.0, 100.0), 50.0)))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(7), circle_path((100.0, 150.0), 50.0)))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(8), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                RemoveLayer(0)
            ]
        ).await;
    });
}

#[test]
fn remove_second_layer() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),
                AddNewLayer(1),
                Layer(1, AddKeyFrame(Duration::from_millis(0))),
                AddNewLayer(2),
                Layer(2, AddKeyFrame(Duration::from_millis(0))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),

                Layer(1, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(102), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(103), BrushProperties::new()))),

                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(3), circle_path((100.0, 100.0), 50.0)))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(4), circle_path((100.0, 150.0), 50.0)))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(5), circle_path((100.0, 200.0), 50.0)))),

                Layer(2, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(104), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(105), BrushProperties::new()))),

                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(6), circle_path((100.0, 100.0), 50.0)))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(7), circle_path((100.0, 150.0), 50.0)))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(8), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                RemoveLayer(1)
            ]
        ).await;
    });
}

#[test]
fn remove_last_layer() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),
                AddNewLayer(1),
                Layer(1, AddKeyFrame(Duration::from_millis(0))),
                AddNewLayer(2),
                Layer(2, AddKeyFrame(Duration::from_millis(0))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),

                Layer(1, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(102), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(103), BrushProperties::new()))),

                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(3), circle_path((100.0, 100.0), 50.0)))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(4), circle_path((100.0, 150.0), 50.0)))),
                Layer(1, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(5), circle_path((100.0, 200.0), 50.0)))),

                Layer(2, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(104), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(105), BrushProperties::new()))),

                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(6), circle_path((100.0, 100.0), 50.0)))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(7), circle_path((100.0, 150.0), 50.0)))),
                Layer(2, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(8), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                RemoveLayer(2)
            ]
        ).await;
    });
}

#[test]
fn add_simple_layer() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_layer_edit_undo(
            vec![
                AddNewLayer(0),
                Layer(0, AddKeyFrame(Duration::from_millis(0))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                AddNewLayer(1),
            ]
        ).await;
    });
}

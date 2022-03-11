use crate::*;
use crate::editor::*;
use crate::storage::*;

use super::undo_element_edits::*;

use flo_stream::*;

use futures::prelude::*;
use futures::future::{select, Either};
use futures_timer::{Delay};

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Data for all of the keyframes in a layer
///
#[derive(PartialEq, Clone)]
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
pub async fn read_all_layers(animation: &impl Animation) -> HashMap<u64, LayerData> {
    let layer_ids   = animation.get_layer_ids();
    let mut layers  = HashMap::new();

    for layer_id in layer_ids {
        let layer_data = read_layer(animation, layer_id).await;
        let layer_data = layer_data.expect("All layers listed in the animation should exist");
        layers.insert(layer_id, layer_data);
    }

    layers
}

///
/// Tests that a layer has the same content before and and after undoing another edit
///
async fn test_layer_edit(setup: Vec<AnimationEdit>, undo_test: Vec<AnimationEdit>) {
    // Create the animation
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    // Perform the setup edits
    animation.edit().publish(Arc::new(vec![
        AnimationEdit::AddNewLayer(0),
        AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(20000))),
    ])).await;
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
    assert!(initial_layers == after_layers);
}

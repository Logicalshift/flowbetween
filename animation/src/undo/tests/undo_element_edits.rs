use crate::*;
use crate::editor::*;
use crate::storage::*;

use flo_stream::*;

use futures::prelude::*;
use futures::future::{select, Either};
use futures_timer::{Delay};

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Tests that frame 0 has the same content after running the edits in undo_test
///
/// This will compare the contents of frame 0 before making the edits and after making the edits and running the corresponding undo actions.
/// The edits should generate at least one undo action, so 0 undo actions is considered a failure.
///
async fn test_element_edit_undo(setup: Vec<AnimationEdit>, undo_test: Vec<AnimationEdit>) {
    // Create the animation
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    // Send the setup actions and wait for them to be accepted
    animation.edit().publish(Arc::new(vec![
        AnimationEdit::AddNewLayer(0),
        AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0)))
    ])).await;
    animation.edit().publish(Arc::new(setup)).await;
    animation.edit().when_empty().await;

    // Read the first frame
    let first_frame         = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
    let initial_elements    = first_frame.vector_elements().unwrap().collect::<Vec<_>>();
    let initial_attachments = initial_elements.iter()
        .map(|elem| elem.id())
        .map(|elem| (elem, first_frame.attached_elements(elem)))
        .collect::<HashMap<_, _>>();

    // The undo action appears when the edits are retired
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

    // Re-read the first frame and compare to the original: should be identical
    let after_frame         = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
    let after_elements      = after_frame.vector_elements().unwrap().collect::<Vec<_>>();
    let after_attachments   = after_elements.iter()
        .map(|elem| elem.id())
        .map(|elem| (elem, after_frame.attached_elements(elem)))
        .collect::<HashMap<_, _>>();

    // Note: we don't read the attachments of group elements recursively so this might miss some differences
    assert!(after_elements == initial_elements);
    assert!(after_attachments == initial_attachments);
}

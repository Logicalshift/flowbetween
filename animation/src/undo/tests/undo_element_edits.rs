use crate::*;
use crate::editor::*;
use crate::storage::*;

use flo_canvas::*;
use flo_stream::*;

use futures::prelude::*;
use futures::executor;
use futures::future::{select, Either};
use futures_timer::{Delay};

use std::sync::*;
use std::time::{Duration};
use std::collections::{HashSet, HashMap};

///
/// Make sure that an element is not attached to an item more than once in a list of edits
///
fn test_no_duplicate_attaches(edits: &Arc<Vec<AnimationEdit>>) {
    use self::AnimationEdit::*;
    use self::ElementEdit::*;

    let mut attached_to = HashMap::new();

    for edit in edits.iter() {
        match edit {
            Element(attachments, AttachTo(attach_to)) => { 
                let element_attachments = attached_to.entry(attach_to).or_insert_with(|| HashSet::new());

                for attachment in attachments.iter() {
                    assert!(!element_attachments.contains(attachment));
                    element_attachments.insert(*attachment);
                }
            }

            Element(attach_to, AddAttachment(attachment)) => { 
                for attach_to in attach_to.iter() {
                    let element_attachments = attached_to.entry(attach_to).or_insert_with(|| HashSet::new());
                    assert!(!element_attachments.contains(attachment));
                    element_attachments.insert(*attachment);
                }
            }

            _ => { }
        }
    }
}

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

    println!("First frame: {}", initial_elements.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

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

    println!("Committed: {}", committed.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));
    println!("Reverse: {}", reverse.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

    // These edits should be equivalent (assuming the example doesn't use unassigned IDs, as the IDs will be assigned at this point)
    assert!(committed == undo_test);

    // The reverse actions should be non-empty (there are ways to create edits that have no effect, but the assumption is the tests won't do this)
    assert!(!reverse.is_empty());

    // Sometimes things like attachments can be added twice to elements: make sure that doesn't happen
    test_no_duplicate_attaches(&reverse);

    // Undo the actions
    animation.edit().publish(Arc::clone(&reverse)).await;

    // Re-read the first frame and compare to the original: should be identical
    let after_frame         = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
    let after_elements      = after_frame.vector_elements().unwrap().collect::<Vec<_>>();
    let after_attachments   = after_elements.iter()
        .map(|elem| elem.id())
        .map(|elem| (elem, after_frame.attached_elements(elem)))
        .collect::<HashMap<_, _>>();

    println!("After undo: {}", after_elements.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

    // Note: we don't read the attachments of group elements recursively so this might miss some differences
    assert!(after_elements == initial_elements);
    assert!(after_attachments == initial_attachments);
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
fn delete_first_element() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_element_edit_undo(
            vec![
                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                Element(vec![ElementId::Assigned(0)], ElementEdit::Delete)
            ]
        ).await;
    });
}

#[test]
fn delete_middle_element() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_element_edit_undo(
            vec![
                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                Element(vec![ElementId::Assigned(1)], ElementEdit::Delete)
            ]
        ).await;
    });
}

#[test]
fn delete_last_element() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_element_edit_undo(
            vec![
                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                Element(vec![ElementId::Assigned(2)], ElementEdit::Delete)
            ]
        ).await;
    });
}

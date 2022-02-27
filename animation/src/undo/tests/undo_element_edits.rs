use crate::*;
use crate::editor::*;
use crate::storage::*;

use flo_canvas::*;
use flo_stream::*;

use futures::prelude::*;
use futures::executor;
use futures::future::{select, Either};
use futures_timer::{Delay};

use std::mem;
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

            Element(detach_from, RemoveAttachment(attachment)) => {
                for detach_from in detach_from.iter() {
                    let element_attachments = attached_to.entry(detach_from).or_insert_with(|| HashSet::new());
                    element_attachments.remove(attachment);
                }
            }

            Element(deleted, Delete) => {
                for delete in deleted {
                    attached_to.remove(delete);

                    for (_, attachments) in attached_to.iter_mut() {
                        attachments.remove(delete);
                    }
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
    // TODO: check for no unassigned element IDs, especially in groups

    // Create the animation
    let in_memory_store = InMemoryStorage::new();
    let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());

    // Send the setup actions and wait for them to be accepted
    animation.edit().publish(Arc::new(vec![
        AnimationEdit::AddNewLayer(0),
        AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(20000))),
    ])).await;
    animation.edit().publish(Arc::new(setup)).await;
    animation.edit().when_empty().await;

    // Read the first frame
    let first_frame         = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
    let initial_elements    = first_frame.vector_elements().unwrap().collect::<Vec<_>>();

    println!("First frame: {}", initial_elements.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

    let initial_subs        = initial_elements.iter().flat_map(|elem| elem.sub_elements().cloned()).collect::<Vec<_>>();

    let initial_attachments = initial_elements.iter()
        .map(|elem| elem.id())
        .map(|elem| (elem, first_frame.attached_elements(elem)))
        .collect::<HashMap<_, _>>();
    let initial_sub_attachs = initial_subs.iter()
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

    println!("Committed: {}", committed.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));
    println!("Reverse: {}", reverse.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

    let commit_frame        = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
    let commit_elements     = commit_frame.vector_elements().unwrap().collect::<Vec<_>>();
    println!("After commit: {}", commit_elements.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

    // These edits should be equivalent (assuming the example doesn't use unassigned IDs, as the IDs will be assigned at this point)
    assert!(committed == undo_test);

    // Sometimes things like attachments can be added twice to elements: make sure that doesn't happen
    test_no_duplicate_attaches(&reverse);

    // The reverse actions should be non-empty (there are ways to create edits that have no effect, but the assumption is the tests won't do this)
    assert!(!reverse.is_empty());

    // Undo the actions
    animation.edit().publish(Arc::clone(&reverse)).await;
    animation.edit().when_empty().await;

    // Re-read the first frame and compare to the original: should be identical
    let after_frame         = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
    let after_elements      = after_frame.vector_elements().unwrap().collect::<Vec<_>>();

    println!("After undo: {}", after_elements.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

    let after_subs          = after_elements.iter().flat_map(|elem| elem.sub_elements().cloned()).collect::<Vec<_>>();

    let after_attachments   = after_elements.iter()
        .map(|elem| elem.id())
        .map(|elem| (elem, first_frame.attached_elements(elem)))
        .collect::<HashMap<_, _>>();

    // Note: we don't read the attachments of group elements recursively so this might miss some differences
    assert!(after_elements == initial_elements);
    assert!(after_attachments == initial_attachments);

    // Fetch a future frame and then re-fetch the 'after' frame to make sure the edits were saved properly to storage as well as the cache
    mem::drop(first_frame);
    mem::drop(after_frame);
    mem::drop(commit_frame);

    animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(20000));

    let after_frame         = animation.get_layer_with_id(0).unwrap().get_frame_at_time(Duration::from_millis(0));
    let after_elements      = after_frame.vector_elements().unwrap().collect::<Vec<_>>();
    let after_attachments   = after_elements.iter()
        .map(|elem| elem.id())
        .map(|elem| (elem, after_frame.attached_elements(elem)))
        .collect::<HashMap<_, _>>();
    let after_sub_attachs   = initial_subs.iter()
        .map(|elem| elem.id())
        .map(|elem| (elem, after_frame.attached_elements(elem)))
        .collect::<HashMap<_, _>>();

    println!("After undo and refetch: {}", after_elements.iter().fold(String::new(), |string, elem| format!("{}\n    {:?}", string, elem)));

    // Note: we don't read the attachments of group elements recursively so this might miss some differences
    assert!(after_elements == initial_elements);
    assert!(after_subs == initial_subs);
    assert!(after_attachments == initial_attachments);
    assert!(after_sub_attachs == initial_sub_attachs);
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

///
/// Generates a circular brush stroke
///
fn circle_brush_stroke(pos: (f64, f64), radius: f64) -> Arc<Vec<RawPoint>> {
    use std::f64;
    let mut points = vec![];

    for p in 0..100 {
        let p = (p as f64) / 100.0;
        let p = (2.0*f64::consts::PI) * p;

        let x = p.sin() * radius + pos.0;
        let y = p.cos() * radius + pos.1;

        points.push(RawPoint::from((x as _, y as _)));
    }

    Arc::new(points)
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

#[test]
fn delete_group() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(3)], ElementEdit::Delete)
            ]
        ).await;
    });
}

#[test]
fn delete_group_first_element() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(0)], ElementEdit::Delete)
            ]
        ).await;
    });
}

#[test]
fn delete_group_middle_element() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(1)], ElementEdit::Delete)
            ]
        ).await;
    });
}

#[test]
fn delete_group_last_element() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(2)], ElementEdit::Delete)
            ]
        ).await;
    });
}

#[test]
fn group_1() {
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
                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ]
        ).await;
    });
}

#[test]
fn group_2() {
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
                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ]
        ).await;
    });
}

#[test]
fn group_3() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1)], ElementEdit::Group(ElementId::Assigned(4), GroupType::Normal))
            ]
        ).await;
    });
}

#[test]
fn group_4() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(4), GroupType::Normal))
            ]
        ).await;
    });
}

#[test]
fn group_5() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1)], ElementEdit::Group(ElementId::Assigned(4), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(4), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ]
        ).await;
    });
}

#[test]
fn ungroup() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(3)], ElementEdit::Ungroup)
            ]
        ).await;
    });
}

#[test]
fn ungroup_2() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal)),
                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1)], ElementEdit::Group(ElementId::Assigned(4), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(4)], ElementEdit::Ungroup)
            ]
        ).await;
    });
}

#[test]
fn set_path() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        let new_path = circle_path((200.0, 200.0), 100.0);

        test_element_edit_undo(
            vec![
                Layer(0, Path(Duration::from_millis(0), PathEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(0), circle_path((100.0, 100.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(1), circle_path((100.0, 150.0), 50.0)))),
                Layer(0, Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(2), circle_path((100.0, 200.0), 50.0)))),
            ],
            vec![
                Element(vec![ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::SetPath(new_path))
            ]
        ).await;
    });
}

#[test]
fn order_in_front() {
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
                Element(vec![ElementId::Assigned(1)], ElementEdit::Order(ElementOrdering::InFront))
            ]
        ).await;
    });
}

#[test]
fn order_behind() {
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
                Element(vec![ElementId::Assigned(1)], ElementEdit::Order(ElementOrdering::Behind))
            ]
        ).await;
    });
}

#[test]
fn order_behind_in_group() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(1)], ElementEdit::Order(ElementOrdering::Behind))
            ]
        ).await;
    });
}

#[test]
fn order_to_top() {
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
                Element(vec![ElementId::Assigned(0)], ElementEdit::Order(ElementOrdering::ToTop))
            ]
        ).await;
    });
}

#[test]
fn order_to_bottom() {
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
                Element(vec![ElementId::Assigned(2)], ElementEdit::Order(ElementOrdering::ToBottom))
            ]
        ).await;
    });
}

#[test]
fn order_before() {
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
                Element(vec![ElementId::Assigned(0)], ElementEdit::Order(ElementOrdering::Before(ElementId::Assigned(2))))
            ]
        ).await;
    });
}

#[test]
fn order_with_parent() {
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

                Element(vec![ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(0)], ElementEdit::Order(ElementOrdering::WithParent(ElementId::Assigned(3))))
            ]
        ).await;
    });
}

#[test]
fn order_to_top_level() {
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

                Element(vec![ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(1)], ElementEdit::Order(ElementOrdering::ToTopLevel))
            ]
        ).await;
    });
}

#[test]
fn convert_to_path() {
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
                Element(vec![ElementId::Assigned(1), ElementId::Assigned(0)], ElementEdit::ConvertToPath)
            ]
        ).await;
    });
}

#[test]
fn convert_to_path_in_group() {
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

                Element(vec![ElementId::Assigned(0), ElementId::Assigned(1), ElementId::Assigned(2)], ElementEdit::Group(ElementId::Assigned(3), GroupType::Normal))
            ],
            vec![
                Element(vec![ElementId::Assigned(1), ElementId::Assigned(0)], ElementEdit::ConvertToPath)
            ]
        ).await;
    });
}

#[test]
fn collide_with_existing_elements_brush_strokes() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_element_edit_undo(
            vec![
                Layer(0, Paint(Duration::from_millis(0), PaintEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Paint(Duration::from_millis(0), PaintEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Assigned(0), circle_brush_stroke((100.0, 100.0), 80.0)))),
                Layer(0, Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Assigned(1), circle_brush_stroke((100.0, 150.0), 80.0)))),
                Layer(0, Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Assigned(2), circle_brush_stroke((100.0, 200.0), 80.0)))),
            ],
            vec![
                Element(vec![ElementId::Assigned(2)], ElementEdit::CollideWithExistingElements)
            ]
        ).await;
    });
}

#[test]
fn collide_with_existing_elements_then_convert_to_path() {
    executor::block_on(async {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;

        test_element_edit_undo(
            vec![
                Layer(0, Paint(Duration::from_millis(0), PaintEdit::SelectBrush(ElementId::Assigned(100), BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
                Layer(0, Paint(Duration::from_millis(0), PaintEdit::BrushProperties(ElementId::Assigned(101), BrushProperties::new()))),

                Layer(0, Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Assigned(0), circle_brush_stroke((100.0, 100.0), 80.0)))),
                Layer(0, Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Assigned(1), circle_brush_stroke((100.0, 150.0), 80.0)))),
                Layer(0, Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Assigned(2), circle_brush_stroke((100.0, 200.0), 80.0)))),
            ],
            vec![
                Element(vec![ElementId::Assigned(2)], ElementEdit::CollideWithExistingElements),
                Element(vec![ElementId::Assigned(2)], ElementEdit::ConvertToPath),
            ]
        ).await;
    });
}

#[test]
fn transform() {
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
                Element(vec![ElementId::Assigned(1), ElementId::Assigned(0)], ElementEdit::Transform(vec![ElementTransform::MoveTo(2.0, 3.0), ElementTransform::SetAnchor(100.0, 100.0), ElementTransform::Rotate(2.0)]))
            ]
        ).await;
    });
}
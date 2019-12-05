use super::*;
use super::tests::*;
use super::db_enum::*;
use super::flo_store::*;
use super::flo_query::*;
use super::motion_path_type::*;

use flo_animation;

use std::time::Duration;

fn test_updates(updates: Vec<DatabaseUpdate>) {
    let core    = core();
    let mut db  = core.db;

    db.update(updates).unwrap();

    assert!(db.stack_is_empty());
}

#[test]
fn smoke_update_canvas_size() {
    test_updates(vec![DatabaseUpdate::UpdateCanvasSize(100.0, 200.0)])
}

#[test]
fn smoke_push_edit_type() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_motion_origin() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::MotionSetOrigin),
        DatabaseUpdate::PushEditLogElementId(0, 1),
        DatabaseUpdate::PushEditLogMotionOrigin(42.0, 24.0),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_motion_type_translate() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::MotionSetOrigin),
        DatabaseUpdate::PushEditLogElementId(0, 1),
        DatabaseUpdate::PushEditLogMotionType(MotionType::Translate),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_motion_element() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::MotionSetOrigin),
        DatabaseUpdate::PushEditLogElementId(0, 1),
        DatabaseUpdate::PushEditLogMotionElement(2),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_element_order_in_front() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::ElementOrderInFront),
        DatabaseUpdate::PushEditLogElementId(0, 1),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_element_order_before() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::ElementOrderBefore),
        DatabaseUpdate::PushEditLogElementId(0, 1),
        DatabaseUpdate::PushEditLogInt(0, 1),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_layer_order() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerSetOrdering),
        DatabaseUpdate::PushEditLogLayer(1),
        DatabaseUpdate::PushEditLogInt(0, 2),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_element_delete() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::ElementDelete),
        DatabaseUpdate::PushEditLogElementId(0, 1),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_motion_path() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::MotionSetOrigin),
        DatabaseUpdate::PushEditLogElementId(0, 1),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushEditLogMotionPath(4),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn adding_edit_type_increases_log_length() {
    let core    = core();
    let mut db  = core.db;

    assert!(db.query_edit_log_length().unwrap() == 0);

    db.update(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame),
        DatabaseUpdate::Pop
    ]).unwrap();

    assert!(db.query_edit_log_length().unwrap() == 1);
}

#[test]
fn can_query_edit_type() {
    let core    = core();
    let mut db  = core.db;

    assert!(db.query_edit_log_length().unwrap() == 0);

    db.update(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame),
        DatabaseUpdate::PushEditLogLayer(3),
        DatabaseUpdate::Pop,
        DatabaseUpdate::PushEditType(EditLogType::SetSize),
        DatabaseUpdate::Pop,
    ]).unwrap();

    let edit_entries = db.query_edit_log_values(0, 1).unwrap();
    assert!(edit_entries.len() == 1);
    assert!(edit_entries[0].edit_type == EditLogType::LayerAddKeyFrame);
    assert!(edit_entries[0].layer_id == Some(3));
    assert!(edit_entries[0].when.is_none());
    assert!(edit_entries[0].brush.is_none());
    assert!(edit_entries[0].brush_properties_id.is_none());

    let edit_entries2 = db.query_edit_log_values(1, 2).unwrap();
    assert!(edit_entries2.len() == 1);
    assert!(edit_entries2[0].edit_type == EditLogType::SetSize);

    let edit_entries3 = db.query_edit_log_values(2, 3).unwrap();
    assert!(edit_entries3.len() == 0);

    let edit_entries4 = db.query_edit_log_values(0, 2).unwrap();
    assert!(edit_entries4.len() == 2);
}

#[test]
fn can_query_motion() {
    let core    = core();
    let mut db  = core.db;

    db.update(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::SetMotionType(1, MotionType::Translate),
        DatabaseUpdate::SetMotionOrigin(1, 100.0, 200.0)
    ]).unwrap();

    let motion_entry = db.query_motion(1);
    let motion_entry = motion_entry.unwrap();

    assert!(motion_entry == Some(MotionEntry { motion_type: MotionType::Translate, origin: Some((100.0, 200.0)) }));
}

#[test]
fn can_query_missing_motion() {
    let core    = core();
    let mut db  = core.db;

    let motion_entry = db.query_motion(1);
    let motion_entry = motion_entry.unwrap();

    assert!(motion_entry == None);
}

#[test]
fn can_query_motion_assigned_elements() {
    let core    = core();
    let mut db  = core.db;

    db.update(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(43),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushVectorElementType(VectorElementType::Motion),
        DatabaseUpdate::PushElementAssignId(1),
        DatabaseUpdate::Pop,
        DatabaseUpdate::PushVectorElementType(VectorElementType::Motion),
        DatabaseUpdate::PushElementAssignId(2),
        DatabaseUpdate::Pop,
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::CreateMotion(2),

        DatabaseUpdate::PushElementIdForAssignedId(42),
        DatabaseUpdate::PushElementIdForAssignedId(1),
        DatabaseUpdate::PushAttachElements(1),
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushElementIdForAssignedId(42),
        DatabaseUpdate::PushElementIdForAssignedId(2),
        DatabaseUpdate::PushAttachElements(1),
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushElementIdForAssignedId(43),
        DatabaseUpdate::PushElementIdForAssignedId(1),
        DatabaseUpdate::PushAttachElements(1),
        DatabaseUpdate::Pop,
    ]).unwrap();

    let element_42      = db.query_vector_element_id(&ElementId::Assigned(42)).unwrap().unwrap();
    let motion_elements = db.query_attached_elements(element_42).unwrap().into_iter()
        .filter_map(|(_, id, element_type)| if element_type == VectorElementType::Motion { id.id() } else { None })
        .collect::<Vec<_>>();

    assert!(motion_elements.iter().any(|item| item == &1));
    assert!(motion_elements.iter().any(|item| item == &2));
    assert!(motion_elements.len() == 2);

    let element_43      = db.query_vector_element_id(&ElementId::Assigned(43)).unwrap().unwrap();
    let motion_elements = db.query_attached_elements(element_43).unwrap().into_iter()
        .filter_map(|(_, id, element_type)| if element_type == VectorElementType::Motion { id.id() } else { None })
        .collect::<Vec<_>>();

    assert!(motion_elements.iter().any(|item| item == &1));
    assert!(motion_elements.len() == 1);

    let element_1           = db.query_vector_element_id(&ElementId::Assigned(1)).unwrap().unwrap();
    let motions_for_element = db.query_elements_with_attachments(element_1).unwrap().into_iter()
        .filter_map(|(_, id, _)| id.id())
        .collect::<Vec<_>>();;

    assert!(motions_for_element.len() == 2);
    assert!(motions_for_element.iter().any(|item| item == &42));
    assert!(motions_for_element.iter().any(|item| item == &43));
}

#[test]
fn can_query_motion_points() {
    let core    = core();
    let mut db  = core.db;

    db.update(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::SetMotionType(1, MotionType::Translate),
        DatabaseUpdate::SetMotionOrigin(1, 100.0, 200.0),

        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(4.0, 5.0, 6.0),
        DatabaseUpdate::PushTimePoint(7.0, 8.0, 9.0),
        DatabaseUpdate::SetMotionPath(1, MotionPathType::Position, 3)
    ]).unwrap();

    let motion_path = db.query_motion_timepoints(1, MotionPathType::Position);
    let motion_path = motion_path.unwrap();

    assert!(motion_path == vec![
        TimePointEntry { x: 1.0, y: 2.0, milliseconds: 3.0 },
        TimePointEntry { x: 4.0, y: 5.0, milliseconds: 6.0 },
        TimePointEntry { x: 7.0, y: 8.0, milliseconds: 9.0 }
    ]);
}

#[test]
fn smoke_pop_edit_log_set_size() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame),
        DatabaseUpdate::PopEditLogSetSize(100.0, 200.0)
    ]);
}

#[test]
fn smoke_push_edit_log_layer() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame),
        DatabaseUpdate::PushEditLogLayer(1),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_edit_log_when() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame),
        DatabaseUpdate::PushEditLogWhen(Duration::from_millis(2000)),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_edit_log_raw_points() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerAddKeyFrame),
        DatabaseUpdate::PushRawPoints(Arc::new(vec![RawPoint::from((0.0, 0.0)), RawPoint::from((1.0, 2.0))])),
        DatabaseUpdate::Pop
    ]);
}

#[test]
fn smoke_push_brush_type() {
    test_updates(vec![
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_ink_brush() {
    test_updates(vec![
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::PushInkBrush(1.0, 2.0, 3.0),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_color_type() {
    test_updates(vec![
        DatabaseUpdate::PushColorType(ColorType::Rgb),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_color_rgb() {
    test_updates(vec![
        DatabaseUpdate::PushColorType(ColorType::Rgb),
        DatabaseUpdate::PushRgb(1.0, 1.0, 1.0),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_color_hsluv() {
    test_updates(vec![
        DatabaseUpdate::PushColorType(ColorType::Hsluv),
        DatabaseUpdate::PushHsluv(20.0, 100.0, 50.0),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_brush_properties() {
    test_updates(vec![
        DatabaseUpdate::PushColorType(ColorType::Hsluv),
        DatabaseUpdate::PushHsluv(20.0, 100.0, 50.0),
        DatabaseUpdate::PushBrushProperties(100.0, 1.0),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_editlog_brush_properties() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerPaintBrushProperties),
        DatabaseUpdate::PushColorType(ColorType::Hsluv),
        DatabaseUpdate::PushHsluv(20.0, 100.0, 50.0),
        DatabaseUpdate::PushBrushProperties(100.0, 1.0),
        DatabaseUpdate::PopEditLogBrushProperties
    ])
}

#[test]
fn smoke_editlog_element_id() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerPaintSelectBrush),
        DatabaseUpdate::PushEditLogElementId(0, 3),
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::PushInkBrush(1.0, 2.0, 3.0),
        DatabaseUpdate::PopEditLogBrush(DrawingStyleType::Erase)
    ])
}

#[test]
fn smoke_editlog_brush() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::LayerPaintSelectBrush),
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::PushInkBrush(1.0, 2.0, 3.0),
        DatabaseUpdate::PopEditLogBrush(DrawingStyleType::Erase)
    ])
}

#[test]
fn smoke_editlog_path() {
    test_updates(vec![
        DatabaseUpdate::PushEditType(EditLogType::ElementSetControlPoints),
        DatabaseUpdate::PushEditLogElementId(0, 3),
        DatabaseUpdate::PushPath(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0), (9.0, 10.0), (11.0, 12.0)]),
        DatabaseUpdate::PushEditLogPath,
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushEditType(EditLogType::ElementSetControlPoints),
        DatabaseUpdate::PushEditLogElementId(0, 4),
        DatabaseUpdate::PushPath(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0), (7.0, 8.0), (9.0, 10.0), (11.0, 12.0)]),
        DatabaseUpdate::PushEditLogPath,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_path_components() {
    test_updates(vec![
        DatabaseUpdate::PushPathComponents(Arc::new(vec![
            PathComponent::Move(PathPoint::new(10.0, 20.0)),
            PathComponent::Line(PathPoint::new(20.0, 30.0)),
            PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
            PathComponent::Close
        ])),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_remove_path_points() {
    test_updates(vec![
        DatabaseUpdate::PushPathComponents(Arc::new(vec![
            PathComponent::Move(PathPoint::new(10.0, 20.0)),
            PathComponent::Line(PathPoint::new(20.0, 30.0)),
            PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
            PathComponent::Close
        ])),
        DatabaseUpdate::PopRemovePathPoints(0..2)
    ])
}

#[test]
fn smoke_insert_path_components() {
    test_updates(vec![
        DatabaseUpdate::PushPathComponents(Arc::new(vec![
            PathComponent::Move(PathPoint::new(10.0, 20.0)),
            PathComponent::Line(PathPoint::new(20.0, 30.0)),
            PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
            PathComponent::Close
        ])),
        DatabaseUpdate::PopInsertPathComponents(1, Arc::new(vec![
            PathComponent::Move(PathPoint::new(10.0, 20.0)),
            PathComponent::Line(PathPoint::new(20.0, 30.0)),
            PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
            PathComponent::Close
        ]))
    ])
}

#[test]
fn smoke_query_path_components() {
    let core    = core();
    let mut db  = core.db;

    db.update(vec![
        DatabaseUpdate::PushPathComponents(Arc::new(vec![
            PathComponent::Move(PathPoint::new(10.0, 20.0)),
            PathComponent::Line(PathPoint::new(20.0, 30.0)),
            PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
            PathComponent::Close
        ])),
        DatabaseUpdate::Pop
    ]).unwrap();

    // Should be path ID 1
    let components = db.query_path_components(1).unwrap();

    assert!(components[0] == PathComponent::Move(PathPoint::new(10.0, 20.0)));
    assert!(components[1] == PathComponent::Line(PathPoint::new(20.0, 30.0)));
    assert!(components[2] == PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)));
    assert!(components[3] == PathComponent::Close);
    assert!(components.len() == 4);
}

#[test]
fn smoke_layer_type() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_delete_layer() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PopDeleteLayer
    ])
}

#[test]
fn smoke_assign_layer() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopDeleteLayer
    ])
}

#[test]
fn smoke_layer_for_assigned_id() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::Pop,
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PopDeleteLayer
    ])
}

#[test]
fn smoke_add_key_frame() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000))
    ])
}

#[test]
fn smoke_remove_key_frame() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PopRemoveKeyFrame(Duration::from_millis(2000))
    ])
}

#[test]
fn smoke_set_layer_name() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PopLayerName("Test".to_string())
    ])
}

#[test]
fn smoke_push_nearest_keyframe() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_vector_element_type() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_vector_element_move_up() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PopVectorElementMove(DbElementMove::Up),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_vector_element_move_to_top() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PopVectorElementMove(DbElementMove::ToTop),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_vector_element_move_down() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PopVectorElementMove(DbElementMove::Down),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_vector_element_move_to_bottom() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PopVectorElementMove(DbElementMove::ToBottom),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_push_vector_element_assign_id() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_pop_vector_brush_element() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushDefinition),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushBrushType(BrushDefinitionType::Ink),
        DatabaseUpdate::PushInkBrush(1.0, 2.0, 3.0),
        DatabaseUpdate::PopVectorBrushElement(DrawingStyleType::Draw),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_pop_vector_brush_properties_element() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushProperties),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushColorType(ColorType::Hsluv),
        DatabaseUpdate::PushHsluv(20.0, 100.0, 50.0),
        DatabaseUpdate::PushBrushProperties(100.0, 1.0),
        DatabaseUpdate::PopVectorBrushPropertiesElement,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_pop_brush_points() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PopBrushPoints(Arc::new(vec![BrushPoint { position: (10.0, 5.0), cp1: (20.0, 20.0), cp2: (30.0, 30.0), width: 10.0 }])),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_create_motion() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::CreateMotion(2)
    ])
}

#[test]
fn smoke_set_motion_type() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::SetMotionType(1, flo_animation::MotionType::Translate)
    ])
}

#[test]
fn smoke_set_motion_origin() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::SetMotionOrigin(1, 20.0, 30.0)
    ])
}

#[test]
fn smoke_set_motion_path() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::SetMotionPath(1, MotionPathType::Position, 4)
    ])
}

#[test]
fn smoke_change_motion_path() {
    test_updates(vec![
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(1.0, 2.0, 3.0),
        DatabaseUpdate::SetMotionPath(1, MotionPathType::Position, 4),

        DatabaseUpdate::PushTimePoint(5.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(6.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(7.0, 2.0, 3.0),
        DatabaseUpdate::PushTimePoint(8.0, 2.0, 3.0),
        DatabaseUpdate::SetMotionPath(1, MotionPathType::Position, 4)
    ])
}

#[test]
fn smoke_attach_elements_to_motion() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushVectorElementType(VectorElementType::Motion),
        DatabaseUpdate::PushElementAssignId(1),
        DatabaseUpdate::Pop,
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::PushElementIdForAssignedId(42),
        DatabaseUpdate::PushElementIdForAssignedId(1),
        DatabaseUpdate::PushAttachElements(1),
        DatabaseUpdate::Pop
    ])
}

#[test]
fn smoke_delete_motion() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushVectorElementType(VectorElementType::Motion),
        DatabaseUpdate::PushElementAssignId(1),
        DatabaseUpdate::Pop,
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::PushElementIdForAssignedId(42),
        DatabaseUpdate::PushElementIdForAssignedId(1),
        DatabaseUpdate::PushAttachElements(1),
        DatabaseUpdate::Pop,

        DatabaseUpdate::DeleteMotion(1)
    ])
}

#[test]
fn smoke_delete_motion_attachment() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushVectorElementType(VectorElementType::Motion),
        DatabaseUpdate::PushElementAssignId(1),
        DatabaseUpdate::Pop,
        DatabaseUpdate::CreateMotion(1),
        DatabaseUpdate::PushElementIdForAssignedId(42),
        DatabaseUpdate::PushElementIdForAssignedId(1),
        DatabaseUpdate::PushAttachElements(1),
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushElementIdForAssignedId(42),
        DatabaseUpdate::PushElementIdForAssignedId(1),
        DatabaseUpdate::PushDetachElements(1),
        DatabaseUpdate::Pop,
    ])
}

#[test]
fn smoke_save_cached_onion_skin() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PopStoreLayerCache(Duration::from_millis(2000), CacheType::OnionSkinLayer, String::from("Test (would be a serialized canvas in a real update)"))
    ])
}

#[test]
fn smoke_delete_cached_onion_skin() {
    test_updates(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::Pop,

        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PopStoreLayerCache(Duration::from_millis(2000), CacheType::OnionSkinLayer, String::from("Test (would be a serialized canvas in a real update)")),

        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PopDeleteLayerCache(Duration::from_millis(2000), CacheType::OnionSkinLayer)
    ])
}

#[test]
fn smoke_delete_vector_element() {
    let core    = core();
    let mut db  = core.db;

    db.update(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
    ]).unwrap();

    assert!(db.stack_is_empty());

    db.update(vec![
        DatabaseUpdate::PushElementIdForAssignedId(42),
        DatabaseUpdate::PopDeleteVectorElement
    ]).unwrap();

    assert!(db.stack_is_empty());
}

#[test]
fn smoke_detach_vector_element() {
    let core    = core();
    let mut db  = core.db;

    db.update(vec![
        DatabaseUpdate::PushLayerType(LayerType::Vector),
        DatabaseUpdate::PushAssignLayer(24),
        DatabaseUpdate::PopAddKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushLayerForAssignedId(24),
        DatabaseUpdate::PushNearestKeyFrame(Duration::from_millis(2000)),
        DatabaseUpdate::PushVectorElementType(VectorElementType::BrushStroke),
        DatabaseUpdate::PushVectorElementTime(Duration::from_millis(2500)),
        DatabaseUpdate::PushElementAssignId(42),
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
        DatabaseUpdate::Pop,
    ]).unwrap();

    assert!(db.stack_is_empty());

    db.update(vec![
        DatabaseUpdate::PushElementIdForAssignedId(42),
        DatabaseUpdate::PopDetachVectorElementFromFrame
    ]).unwrap();

    assert!(db.stack_is_empty());
}

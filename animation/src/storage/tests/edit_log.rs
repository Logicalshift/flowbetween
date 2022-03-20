use super::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

#[test]
fn draw_brush_strokes() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);
}

#[test]
fn draw_brush_strokes_in_future() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(440), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(440), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(440), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(450), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(500), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);
}

#[test]
fn edit_brush_strokes() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(100), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),

        AnimationEdit::Element(vec![ElementId::Assigned(100)], ElementEdit::SetControlPoints(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)], Duration::from_millis(442)))
    ]);
}

#[test]
fn read_brush_strokes_from_edit_log() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
    ]);

    let edit_log        = anim.read_edit_log(0..7);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    assert!(edits.len() == 7);
    assert!(edits[0] == AnimationEdit::AddNewLayer(2));
    assert!(edits[1] == AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))));
    assert!(match edits[2] {
        AnimationEdit::Layer(2, LayerEdit::Paint(_when, PaintEdit::SelectBrush(
                ElementId::Assigned(_element_id),
                BrushDefinition::Ink(ref ink_defn),
                BrushDrawingStyle::Draw
            )
        ))  => ink_defn == &InkDefinition::default(),
        _   => false
    });
    assert!(match edits[3] {
        AnimationEdit::Layer(2, LayerEdit::Paint(_when, PaintEdit::
            BrushProperties(ElementId::Assigned(_element_id), ref brush_properties)))
                => brush_properties == &BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 },
            _ => false
    });
    assert!(match edits[6] {
        AnimationEdit::Layer(2, LayerEdit::Paint(ref when, PaintEdit::BrushStroke(ElementId::Assigned(_), ref points)))
                        => points == &Arc::new(vec![
                            RawPoint::from((10.0, 10.0)),
                            RawPoint::from((20.0, 5.0))
                        ]) && when == &Duration::from_millis(442),
                _       => false
    });
}

#[test]
fn read_element_delete_from_edit_log() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(50))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(126), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(127), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::BrushStroke(ElementId::Assigned(128), Arc::new(vec![
                    RawPoint::from((10.0, 10.0)),
                    RawPoint::from((20.0, 5.0))
                ])))),
        AnimationEdit::Element(vec![ElementId::Assigned(127), ElementId::Assigned(128), ElementId::Assigned(126)], ElementEdit::Delete)
    ]);

    let edit_log        = anim.read_edit_log(7..8);
    let edit_log        = edit_log.collect();
    let edits: Vec<_>   = executor::block_on(edit_log);

    assert!(edits.len() == 1);
    assert!(edits[0] == AnimationEdit::Element(vec![ElementId::Assigned(127), ElementId::Assigned(128), ElementId::Assigned(126)], ElementEdit::Delete));
}

#[test]
fn draw_shape() {
    let anim = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::CreateShape(ElementId::Unassigned, 0.5, Shape::Circle { center: (100.0, 110.0), point: (130.0, 140.0) }))),
    ]);
}

#[test]
fn will_assign_element_ids() {
    let animation = create_animation();

    // Perform some edits on the animation with an unassigned element ID
    animation.perform_edits(vec![
        AnimationEdit::AddNewLayer(0),
        AnimationEdit::Layer(0, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(0, LayerEdit::Paint(Duration::from_millis(0), PaintEdit::SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            )
        )),
        AnimationEdit::Layer(0, LayerEdit::Paint(Duration::from_millis(0), PaintEdit::BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
        AnimationEdit::Layer(0, LayerEdit::Paint(Duration::from_millis(0), PaintEdit::BrushStroke(ElementId::Unassigned, Arc::new(vec![
                RawPoint::from((10.0, 10.0)),
                RawPoint::from((20.0, 5.0))
            ]))))
    ]);

    // Element ID should be assigned if we read the log back
    let edit_log            = animation.read_edit_log(4..5);
    let edit_log            = edit_log.collect();

    let paint_edit: Vec<_>  = executor::block_on(edit_log);

    // Should be able to find the paint edit here
    assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, _)) => true, _ => false });

    // Element ID should be assigned
    assert!(match &paint_edit[0] { &AnimationEdit::Layer(0, LayerEdit::Paint(_, PaintEdit::BrushStroke(ElementId::Assigned(_), _))) => true, _ => false });
}

#[test]
fn create_vector_element() {
    let anim                = create_animation();
    let mut test_element    = Vector::BrushDefinition(BrushDefinitionElement::default());

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::CreateElement(Duration::from_millis(0), ElementId::Assigned(42), test_element.clone())),
    ]);

    // Readback
    let edit_log            = anim.read_edit_log(2..3);
    let edit_log            = edit_log.collect();

    let create_element: Vec<_>  = executor::block_on(edit_log);
    test_element.set_id(ElementId::Assigned(42));

    println!("{:?}", create_element);

    assert!(create_element.len() == 1);
    assert!(create_element[0] == AnimationEdit::Layer(2, LayerEdit::CreateElement(Duration::from_millis(0), ElementId::Assigned(42), test_element.clone())));
}

#[test]
fn undo_begin_action() {
    let anim                = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Undo(UndoEdit::BeginAction)
    ]);

    // Readback
    let edit_log            = anim.read_edit_log(2..3);
    let edit_log            = edit_log.collect();

    let undo_action: Vec<_> = executor::block_on(edit_log);

    assert!(undo_action.len() == 1);
    assert!(undo_action[0] == AnimationEdit::Undo(UndoEdit::BeginAction));
}

#[test]
fn undo_prepare_to_undo_not_written() {
    let anim                = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Undo(UndoEdit::PrepareToUndo("Test".to_string()))
    ]);

    // Readback
    let edit_log_len        = anim.get_num_edits();
    assert!(edit_log_len == 2);

    let edit_log            = anim.read_edit_log(0..3);
    let edit_log            = edit_log.collect();

    let edit_log: Vec<_>    = executor::block_on(edit_log);

    assert!(edit_log.len() == 2);
}

#[test]
fn undo_perform_undo_not_written() {
    let anim                = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Undo(UndoEdit::PerformUndo { original_actions: Arc::new(vec![]), undo_actions: Arc::new(vec![]) })
    ]);

    // Readback
    let edit_log_len        = anim.get_num_edits();
    assert!(edit_log_len == 2);

    let edit_log            = anim.read_edit_log(0..3);
    let edit_log            = edit_log.collect();

    let edit_log: Vec<_>    = executor::block_on(edit_log);

    assert!(edit_log.len() == 2);
}

#[test]
fn undo_perform_undo_rolls_back_log() {
    let anim                = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);
    anim.perform_edits(vec![
        AnimationEdit::Undo(UndoEdit::PerformUndo { 
            original_actions:   Arc::new(vec![AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0)))]), 
            undo_actions:       Arc::new(vec![]) 
        })
    ]);

    // Readback
    let edit_log_len        = anim.get_num_edits();
    assert!(edit_log_len == 1);
    
    let edit_log            = anim.read_edit_log(0..3);
    let edit_log            = edit_log.collect();

    let edit_log: Vec<_>    = executor::block_on(edit_log);

    assert!(edit_log.len() == 1);
}

#[test]
fn undo_bad_undo_does_not_roll_back_log() {
    let anim                = create_animation();

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
    ]);

    // Actions in original_actions do not match the actions on top of the edit log so the undo will not be performed
    anim.perform_edits(vec![
        AnimationEdit::Undo(UndoEdit::PerformUndo { 
            original_actions:   Arc::new(vec![AnimationEdit::AddNewLayer(2)]), 
            undo_actions:       Arc::new(vec![]) 
        })
    ]);

    // Readback
    let edit_log_len        = anim.get_num_edits();
    assert!(edit_log_len == 2);
    
    let edit_log            = anim.read_edit_log(0..3);
    let edit_log            = edit_log.collect();

    let edit_log: Vec<_>    = executor::block_on(edit_log);

    assert!(edit_log.len() == 2);
}

use super::*;

use flo_canvas::*;

use std::sync::*;
use std::time::Duration;

#[test]
fn fetch_element_by_id() {
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
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
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
    ]);

    let layer = anim.get_layer_with_id(2).unwrap();

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let element127          = frame.element_with_id(ElementId::Assigned(127));

        assert!(match element127 {
            Some(Vector::BrushStroke(ref brush_stroke)) => Some(brush_stroke.id()),
            _ => None
        } == Some(ElementId::Assigned(127)));
    }

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(60));
        let elements: Vec<_>    = frame.vector_elements().unwrap().collect();

        assert!(frame.time_index() == Duration::from_millis(60));
        assert!(elements.len() == 0);
    }
}

#[test]
fn read_frame_after_edits() {
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
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Unassigned, BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
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
    ]);

    let layer = anim.get_layer_with_id(2).unwrap();

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let elements: Vec<_>    = frame.vector_elements().unwrap().collect();

        assert!(frame.time_index() == Duration::from_millis(442));
        assert!(elements.len() == 3);

        assert!(match &elements[1] {
            &Vector::BrushStroke(ref brush_stroke) => Some(brush_stroke.points()),
            _ => None
        }.is_some());

        assert!(match &elements[1] {
            &Vector::BrushStroke(ref brush_stroke) => Some(brush_stroke.id()),
            _ => None
        } == Some(ElementId::Assigned(127)));

        // All our elements should have the brush properties attached to them
        let attachments = frame.attached_elements(ElementId::Assigned(126));
        assert!(attachments.len() != 0);
        assert!(attachments.len() == 2);

        let attachments = frame.attached_elements(ElementId::Assigned(127));
        assert!(attachments.len() != 0);
        assert!(attachments.len() == 2);

        let attachments = frame.attached_elements(ElementId::Assigned(128));
        assert!(attachments.len() != 0);
        assert!(attachments.len() == 2);
    }

    {
        let frame               = layer.get_frame_at_time(Duration::from_millis(60));
        let elements: Vec<_>    = frame.vector_elements().unwrap().collect();

        assert!(frame.time_index() == Duration::from_millis(60));
        assert!(elements.len() == 0);
    }
}

#[test]
fn delete_element() {
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
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Assigned(125), BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
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
    ]);

    {
        let layer               = anim.get_layer_with_id(2).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));

        let all_elements        = frame.vector_elements().unwrap()
            .map(|elem| elem.id())
            .collect::<Vec<_>>();
        assert!(all_elements == vec![ElementId::Assigned(126), ElementId::Assigned(127), ElementId::Assigned(128)]);
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(127)], ElementEdit::Delete)
    ]);

    {
        let layer               = anim.get_layer_with_id(2).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let element127          = frame.element_with_id(ElementId::Assigned(127));

        assert!(match element127 {
            Some(Vector::BrushStroke(ref brush_stroke)) => Some(brush_stroke.id()),
            _ => None
        } == None);

        let all_elements        = frame.vector_elements().unwrap()
            .map(|elem| elem.id())
            .collect::<Vec<_>>();
        assert!(all_elements == vec![ElementId::Assigned(126), ElementId::Assigned(128)]);
    }
}

#[test]
fn delete_first_element() {
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
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Assigned(125), BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
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
    ]);

    {
        let layer               = anim.get_layer_with_id(2).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));

        let all_elements        = frame.vector_elements().unwrap()
            .map(|elem| elem.id())
            .collect::<Vec<_>>();
        assert!(all_elements == vec![ElementId::Assigned(126), ElementId::Assigned(127), ElementId::Assigned(128)]);
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(126)], ElementEdit::Delete)
    ]);

    {
        let layer               = anim.get_layer_with_id(2).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let element126          = frame.element_with_id(ElementId::Assigned(126));

        assert!(match element126 {
            Some(Vector::BrushStroke(ref brush_stroke)) => Some(brush_stroke.id()),
            _ => None
        } == None);

        let all_elements        = frame.vector_elements().unwrap()
            .map(|elem| elem.id())
            .collect::<Vec<_>>();
        assert!(all_elements == vec![ElementId::Assigned(127), ElementId::Assigned(128)]);
    }
}

#[test]
fn delete_last_element() {
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
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Assigned(125), BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
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
    ]);

    {
        let layer               = anim.get_layer_with_id(2).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));

        let all_elements        = frame.vector_elements().unwrap()
            .map(|elem| elem.id())
            .collect::<Vec<_>>();
        assert!(all_elements == vec![ElementId::Assigned(126), ElementId::Assigned(127), ElementId::Assigned(128)]);
    }

    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(128)], ElementEdit::Delete)
    ]);

    {
        let layer               = anim.get_layer_with_id(2).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let element128          = frame.element_with_id(ElementId::Assigned(128));

        assert!(match element128 {
            Some(Vector::BrushStroke(ref brush_stroke)) => Some(brush_stroke.id()),
            _ => None
        } == None);

        let all_elements        = frame.vector_elements().unwrap()
            .map(|elem| elem.id())
            .collect::<Vec<_>>();
        assert!(all_elements == vec![ElementId::Assigned(126), ElementId::Assigned(127)]);
    }
}

#[test]
fn detach_element() {
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
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::
            BrushProperties(ElementId::Assigned(125), BrushProperties { color: Color::Rgba(0.5, 0.2, 0.7, 1.0), opacity: 1.0, size: 32.0 }))),
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
    ]);

    {
        let layer               = anim.get_layer_with_id(2).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));

        let all_elements        = frame.vector_elements().unwrap()
            .map(|elem| elem.id())
            .collect::<Vec<_>>();
        assert!(all_elements == vec![ElementId::Assigned(126), ElementId::Assigned(127), ElementId::Assigned(128)]);
    }

    // Detach is similar to delete but the element is not actually removed from anywhere other than the frame itself
    anim.perform_edits(vec![
        AnimationEdit::Element(vec![ElementId::Assigned(127)], ElementEdit::DetachFromFrame)
    ]);

    {
        let layer               = anim.get_layer_with_id(2).unwrap();
        let frame               = layer.get_frame_at_time(Duration::from_millis(442));
        let element127          = frame.element_with_id(ElementId::Assigned(127));

        assert!(match element127 {
            Some(Vector::BrushStroke(ref brush_stroke)) => Some(brush_stroke.id()),
            _ => None
        } == None);

        let all_elements        = frame.vector_elements().unwrap()
            .map(|elem| elem.id())
            .collect::<Vec<_>>();
        assert!(all_elements == vec![ElementId::Assigned(126), ElementId::Assigned(128)]);
    }
}

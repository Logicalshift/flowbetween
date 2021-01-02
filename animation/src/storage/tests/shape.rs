use super::*;

use std::time::Duration;

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
        AnimationEdit::Layer(2, LayerEdit::Paint(Duration::from_millis(442), PaintEdit::CreateShape(ElementId::Unassigned, 0.5, Shape::Circle { center: (100.0, 110.0), point: (130.0, 140.0) }))),
    ]);

    // Layer should contain two shape elements
    let layer               = anim.get_layer_with_id(2).unwrap();
    let frame               = layer.get_frame_at_time(Duration::from_millis(442));

    assert!(frame.vector_elements().is_some());
    assert!(frame.vector_elements().unwrap().count() > 0);
    assert!(frame.vector_elements().unwrap().count() == 2);
}

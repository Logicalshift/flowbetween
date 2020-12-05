use super::*;

use std::sync::*;
use std::time::Duration;

#[test]
fn cut_square_into_doughnut() {
    let anim = create_animation();

    // Create a square
    let square = Arc::new(vec![
        PathComponent::Move(PathPoint::new(100.0, 100.0)),
        PathComponent::Line(PathPoint::new(200.0, 100.0)),
        PathComponent::Line(PathPoint::new(200.0, 200.0)),
        PathComponent::Line(PathPoint::new(100.0, 200.0)),
        PathComponent::Line(PathPoint::new(100.0, 100.0))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::SelectBrush(
                ElementId::Assigned(1),
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            ))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(2), BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(3), square)))
    ]);

    // Layer should contain the square
    let layer       = anim.get_layer_with_id(2).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() == 1);
    assert!(elements[0].id() == ElementId::Assigned(3));

    // Cut out its center
    let center_square = Arc::new(vec![
        PathComponent::Move(PathPoint::new(125.0, 125.0)),
        PathComponent::Line(PathPoint::new(175.0, 125.0)),
        PathComponent::Line(PathPoint::new(175.0, 175.0)),
        PathComponent::Line(PathPoint::new(125.0, 175.0)),
        PathComponent::Line(PathPoint::new(125.0, 125.0))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Cut { path: center_square, when: Duration::from_millis(0), inside_group: ElementId::Assigned(100), outside_group: ElementId::Assigned(101) })
    ]);

    // Layer should contain 2 groups with the inside and outside elements in it
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() != 1);
    assert!(elements.len() == 2);

    assert!(elements[0].id() == ElementId::Assigned(100));
    assert!(elements[1].id() == ElementId::Assigned(101));
}

#[test]
fn include_entire_path_in_cut() {
    let anim = create_animation();

    // Create a square
    let square = Arc::new(vec![
        PathComponent::Move(PathPoint::new(125.0, 125.0)),
        PathComponent::Line(PathPoint::new(175.0, 125.0)),
        PathComponent::Line(PathPoint::new(175.0, 175.0)),
        PathComponent::Line(PathPoint::new(125.0, 175.0)),
        PathComponent::Line(PathPoint::new(125.0, 125.0))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::SelectBrush(
                ElementId::Assigned(1),
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            ))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(2), BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(3), square)))
    ]);

    // Layer should contain the square
    let layer       = anim.get_layer_with_id(2).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() == 1);
    assert!(elements[0].id() == ElementId::Assigned(3));

    // Include it in a cut
    let center_square = Arc::new(vec![
        PathComponent::Move(PathPoint::new(100.0, 100.0)),
        PathComponent::Line(PathPoint::new(200.0, 100.0)),
        PathComponent::Line(PathPoint::new(200.0, 200.0)),
        PathComponent::Line(PathPoint::new(100.0, 200.0)),
        PathComponent::Line(PathPoint::new(100.0, 100.0))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Cut { path: center_square, when: Duration::from_millis(0), inside_group: ElementId::Assigned(100), outside_group: ElementId::Assigned(101) })
    ]);

    // Layer should contain 2 groups with the inside and outside elements in it
    let layer       = anim.get_layer_with_id(2).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // This just creates a group containing the square that was cut out
    assert!(elements.len() == 1);
    assert!(elements[0].id() == ElementId::Assigned(100));
}

#[test]
fn include_nothing_in_cut() {
    let anim = create_animation();

    // Create a square
    let square = Arc::new(vec![
        PathComponent::Move(PathPoint::new(125.0, 125.0)),
        PathComponent::Line(PathPoint::new(175.0, 125.0)),
        PathComponent::Line(PathPoint::new(175.0, 175.0)),
        PathComponent::Line(PathPoint::new(125.0, 175.0)),
        PathComponent::Line(PathPoint::new(125.0, 125.0))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(2),
        AnimationEdit::Layer(2, LayerEdit::AddKeyFrame(Duration::from_millis(0))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::SelectBrush(
                ElementId::Assigned(1),
                BrushDefinition::Ink(InkDefinition::default()),
                BrushDrawingStyle::Draw
            ))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::BrushProperties(ElementId::Assigned(2), BrushProperties::new()))),
        AnimationEdit::Layer(2, LayerEdit::Path(Duration::from_millis(0), PathEdit::CreatePath(ElementId::Assigned(3), square)))
    ]);

    // Layer should contain the square
    let layer       = anim.get_layer_with_id(2).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    assert!(elements.len() == 1);
    assert!(elements[0].id() == ElementId::Assigned(3));

    // Cut an empty region of the canvas
    let center_square = Arc::new(vec![
        PathComponent::Move(PathPoint::new(300.0, 300.0)),
        PathComponent::Line(PathPoint::new(400.0, 300.0)),
        PathComponent::Line(PathPoint::new(400.0, 400.0)),
        PathComponent::Line(PathPoint::new(300.0, 400.0)),
        PathComponent::Line(PathPoint::new(300.0, 300.0))
    ]);

    anim.perform_edits(vec![
        AnimationEdit::Layer(2, LayerEdit::Cut { path: center_square, when: Duration::from_millis(0), inside_group: ElementId::Assigned(100), outside_group: ElementId::Assigned(101) })
    ]);

    // Layer should contain 2 groups with the inside and outside elements in it
    let layer       = anim.get_layer_with_id(2).unwrap();
    let frame       = layer.get_frame_at_time(Duration::from_millis(0));
    let elements    = frame.vector_elements().unwrap().collect::<Vec<_>>();

    // This should leave the canvas as it was before the cut
    assert!(elements.len() == 1);
    assert!(elements[0].id() == ElementId::Assigned(3));
}

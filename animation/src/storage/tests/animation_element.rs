use super::*;

use flo_curves::*;
use flo_curves::arc::*;
use flo_curves::bezier::path::*;
use flo_canvas_animation::description::*;

use std::sync::*;
use std::time::Duration;

#[test]
fn create_animation_element() {
    use self::LayerEdit::*;

    let anim                = create_animation();

    let circle              = Circle::new(Coord2(100.0, 100.0), 50.0).to_path::<SimpleBezierPath>();
    let animation_region    = RegionDescription(vec![circle.into()], EffectDescription::Sequence(vec![]));

    anim.perform_edits(vec![
        AnimationEdit::AddNewLayer(24),
        AnimationEdit::Layer(24, LayerEdit::AddKeyFrame(Duration::from_millis(300))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::SelectBrush(ElementId::Unassigned, BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::BrushProperties(ElementId::Unassigned, BrushProperties::new()))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Assigned(100), Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ])))),
        AnimationEdit::Layer(24, CreateAnimation(Duration::from_millis(300), ElementId::Assigned(101), animation_region))
    ]);

    let layer               = anim.get_layer_with_id(24).unwrap();
    let frame               = layer.get_frame_at_time(Duration::from_millis(300));

    assert!(frame.vector_elements().is_some());
    assert!(frame.vector_elements().unwrap().count() > 0);

    let element100          = frame.element_with_id(ElementId::Assigned(100)).unwrap();
    let element101          = frame.element_with_id(ElementId::Assigned(101)).unwrap();

    if let Vector::Path(_path) = element100 {
        assert!(true);
    } else {
        // Not a path
        assert!(false);
    }

    if let Vector::AnimationRegion(_animation) = element101 {
        assert!(true);
    } else {
        // Not a path
        assert!(false);
    }
}

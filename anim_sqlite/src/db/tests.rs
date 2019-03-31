use super::*;
use super::flo_query::*;

use flo_animation;
use flo_animation::LayerEdit::*;
use flo_animation::PaintEdit::*;

use std::time::Duration;

#[test]
fn can_create_new_database() {
    let db = AnimationDb::new();
    assert!(db.retrieve_and_clear_error().is_none());
}

pub fn core() -> AnimationDbCore<FloSqlite> {
    let connection = Connection::open_in_memory().unwrap();
    FloSqlite::setup(&connection).unwrap();

    let core = AnimationDbCore::new(connection);
    core
}

#[test]
fn initial_length_is_two_minutes() {
    assert!(core().db.query_duration().unwrap() == Duration::from_secs(120));
}

#[test]
fn initial_frame_rate_is_30fps() {
    assert!(core().db.query_frame_length().unwrap() == Duration::new(0, 33_333_333));
}

#[test]
fn insert_set_size() {
    core().insert_edits(&[AnimationEdit::SetSize(1980.0, 1080.0)]).unwrap();
}

#[test]
fn insert_add_new_layer() {
    core().insert_edits(&[AnimationEdit::AddNewLayer(24)]).unwrap();
}

#[test]
fn remove_layer() {
    core().insert_edits(&[AnimationEdit::RemoveLayer(24)]).unwrap();
}

#[test]
fn add_key_frame() {
    core().insert_edits(&[AnimationEdit::Layer(24, AddKeyFrame(Duration::from_millis(300)))]).unwrap();
}

#[test]
fn remove_key_frame() {
    core().insert_edits(&[AnimationEdit::Layer(24, RemoveKeyFrame(Duration::from_millis(300)))]).unwrap();
}

#[test]
fn set_layer_name() {
    core().insert_edits(&[AnimationEdit::Layer(24, SetName("Some layer".to_string()))]).unwrap();
}

#[test]
fn select_brush() {
    core().insert_edits(&[AnimationEdit::Layer(24, 
        Paint(Duration::from_millis(300), 
            SelectBrush(
                ElementId::Unassigned,
                BrushDefinition::Ink(InkDefinition::default()), 
                BrushDrawingStyle::Draw
            )
        )
    )]).unwrap();
}

#[test]
fn brush_properties() {
    core().insert_edits(&[AnimationEdit::Layer(24,
        Paint(Duration::from_millis(300),
            BrushProperties(ElementId::Unassigned, flo_animation::BrushProperties::new())
        )
    )]).unwrap();
}

#[test]
fn brush_stroke() {
    core().insert_edits(&[AnimationEdit::Layer(24,
        Paint(Duration::from_millis(300),
            BrushStroke(ElementId::Unassigned, Arc::new(vec![
                RawPoint::from((0.0, 0.0)),
                RawPoint::from((10.0, 0.0)),
                RawPoint::from((10.0, 10.0)),
                RawPoint::from((0.0, 10.0)),
                RawPoint::from((0.0, 0.0))
            ]))
        )
    )]).unwrap();
}

#[test]
fn attach_element() {
    core().insert_edits(&[AnimationEdit::Layer(24,
        Paint(Duration::from_millis(300),
            BrushStroke(ElementId::Assigned(128), Arc::new(vec![
                RawPoint::from((0.0, 0.0)),
                RawPoint::from((10.0, 0.0)),
                RawPoint::from((10.0, 10.0)),
                RawPoint::from((0.0, 10.0)),
                RawPoint::from((0.0, 0.0))
            ]))
        )),

        AnimationEdit::Layer(24,
        Paint(Duration::from_millis(300),
            BrushStroke(ElementId::Assigned(129), Arc::new(vec![
                RawPoint::from((0.0, 0.0)),
                RawPoint::from((10.0, 0.0)),
                RawPoint::from((10.0, 10.0)),
                RawPoint::from((0.0, 10.0)),
                RawPoint::from((0.0, 0.0))
            ]))
        )),

        AnimationEdit::Element(vec![ElementId::Assigned(129)], ElementEdit::AttachTo(ElementId::Assigned(128)))
    ]).unwrap();
}

#[test]
fn create_path() {
    core().insert_edits(&[
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::SelectBrush(ElementId::Unassigned, BrushDefinition::Ink(InkDefinition::default()), BrushDrawingStyle::Draw))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::BrushProperties(ElementId::Unassigned, flo_animation::BrushProperties::new()))),
        AnimationEdit::Layer(24, Path(Duration::from_millis(300),
            PathEdit::CreatePath(ElementId::Unassigned, Arc::new(vec![
                PathComponent::Move(PathPoint::new(10.0, 20.0)),
                PathComponent::Line(PathPoint::new(20.0, 30.0)),
                PathComponent::Bezier(PathPoint::new(40.0, 40.0), PathPoint::new(30.0, 30.0), PathPoint::new(20.0, 20.0)),
                PathComponent::Close
            ]))))
    ]).unwrap();
}

#[test]
fn translate_motion() {
    let start_point = TimePoint::new(10.0, 20.0, Duration::from_millis(0));
    let end_point   = TimePoint::new(500.0, 400.0, Duration::from_millis(2000));

    core().insert_edits(&[
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetOrigin(30.0, 40.0)),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetPath(TimeCurve::new(start_point, end_point))),
    ]).unwrap();
}

#[test]
fn delete_translate_motion() {
    let start_point = TimePoint::new(10.0, 20.0, Duration::from_millis(0));
    let end_point   = TimePoint::new(500.0, 400.0, Duration::from_millis(2000));

    core().insert_edits(&[
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::Create),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetType(MotionType::Translate)),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetOrigin(30.0, 40.0)),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::SetPath(TimeCurve::new(start_point, end_point))),
        AnimationEdit::Motion(ElementId::Assigned(1), MotionEdit::Delete)
    ]).unwrap();
}

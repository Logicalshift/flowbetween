use super::*;

use animation;
use animation::*;
use animation::LayerEdit::*;
use animation::PaintEdit::*;

#[test]
fn can_create_new_database() {
    let db = AnimationDb::new();
    assert!(db.retrieve_and_clear_error().is_none());
}

#[test]
fn can_read_default_enum() {
    let mut db = AnimationDbCore::new(Connection::open_in_memory().unwrap());
    db.setup().unwrap();

    let edit_enum = EditLogEnumValues::new(&db.sqlite);

    assert!(edit_enum.layer_paint_select_brush == 5);
}

fn core() -> AnimationDbCore {
    let mut core = AnimationDbCore::new(Connection::open_in_memory().unwrap());
    core.setup().unwrap();
    core
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
fn select_brush() {
    core().insert_edits(&[AnimationEdit::Layer(24, 
        Paint(Duration::from_millis(300), 
            SelectBrush(
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
            BrushProperties(animation::BrushProperties::new())
        )
    )]).unwrap();
}

#[test]
fn brush_stroke() {
    core().insert_edits(&[AnimationEdit::Layer(24,
        Paint(Duration::from_millis(300),
            BrushStroke(Arc::new(vec![
                RawPoint::from((0.0, 0.0)),
                RawPoint::from((10.0, 0.0)),
                RawPoint::from((10.0, 10.0)),
                RawPoint::from((0.0, 10.0)),
                RawPoint::from((0.0, 0.0))
            ]))
        )
    )]).unwrap();
}

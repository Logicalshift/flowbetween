use super::*;

#[test]
fn can_create_new_database() {
    let db = AnimationDb::new();
    assert!(db.retrieve_and_clear_error().is_none());
}

#[test]
fn can_read_default_enum() {
    let mut db = AnimationDbCore::new(Connection::open_in_memory().unwrap());
    db.setup();

    let edit_enum = EditLogEnumValues::new(&db.sqlite);

    assert!(edit_enum.layer_paint_select_brush == 5);
}

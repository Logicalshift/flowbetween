use super::*;
use super::super::point::*;

use super::super::brush::*;
use super::super::layer::*;
use super::super::property::*;
use super::super::queries::*;
use super::super::shape::*;
use rusqlite::*;

/// Helper: returns the ShapeGuids for shapes on a layer, ordered by OrderIdx
fn shapes_on_layer(canvas: &SqliteCanvas, layer_id: CanvasLayerId) -> Vec<String> {
    let layer_idx   = canvas.sqlite.query_one::<i64, _, _>("SELECT LayerId FROM Layers WHERE LayerGuid = ?", [layer_id.to_string()], |row| row.get(0)).unwrap();
    let mut stmt    = canvas.sqlite.prepare("SELECT s.ShapeGuid FROM ShapeLayers sl JOIN Shapes s ON sl.ShapeId = s.ShapeId WHERE sl.LayerId = ? ORDER BY sl.OrderIdx ASC").unwrap();
    let rows        = stmt.query_map(params![layer_idx], |row| row.get::<_, String>(0)).unwrap();
    rows.map(|r| r.unwrap()).collect()
}

/// Helper: returns the ShapeGuids for shapes in a group, ordered by OrderIdx
fn shapes_in_group(canvas: &SqliteCanvas, parent_shape_id: CanvasShapeId) -> Vec<String> {
    let parent_idx  = canvas.sqlite.query_one::<i64, _, _>("SELECT ShapeId FROM Shapes WHERE ShapeGuid = ?", [parent_shape_id.to_string()], |row| row.get(0)).unwrap();
    let mut stmt    = canvas.sqlite.prepare("SELECT s.ShapeGuid FROM ShapeGroups sg JOIN Shapes s ON sg.ShapeId = s.ShapeId WHERE sg.ParentShapeId = ? ORDER BY sg.OrderIdx ASC").unwrap();
    let rows        = stmt.query_map(params![parent_idx], |row| row.get::<_, String>(0)).unwrap();
    rows.map(|r| r.unwrap()).collect()
}

/// Helper: returns the BrushGuids associated with a shape, ordered by OrderIdx
fn brushes_on_shape(canvas: &SqliteCanvas, shape_id: CanvasShapeId) -> Vec<String> {
    let shape_idx   = canvas.sqlite.query_one::<i64, _, _>("SELECT ShapeId FROM Shapes WHERE ShapeGuid = ?", [shape_id.to_string()], |row| row.get(0)).unwrap();
    let mut stmt    = canvas.sqlite.prepare("SELECT b.BrushGuid FROM ShapeBrushes sb JOIN Brushes b ON sb.BrushId = b.BrushId WHERE sb.ShapeId = ? ORDER BY sb.OrderIdx ASC").unwrap();
    let rows        = stmt.query_map(params![shape_idx], |row| row.get::<_, String>(0)).unwrap();
    rows.map(|r| r.unwrap()).collect()
}

/// Helper: directly inserts a brush into the Brushes table (since AddBrush is not yet implemented)
fn insert_brush(canvas: &SqliteCanvas, brush_id: CanvasBrushId) {
    canvas.sqlite.execute("INSERT INTO Brushes (BrushGuid) VALUES (?)", params![brush_id.to_string()]).unwrap();
}

fn test_rect() -> CanvasShape {
    CanvasShape::Rectangle(CanvasRectangle { min: CanvasPoint { x: 0.0, y: 0.0 }, max: CanvasPoint { x: 10.0, y: 10.0 } })
}

fn test_ellipse() -> CanvasShape {
    CanvasShape::Ellipse(CanvasEllipse { min: CanvasPoint { x: 0.0, y: 0.0 }, max: CanvasPoint { x: 5.0, y: 5.0 }, direction: CanvasPoint { x: 1.0, y: 0.0 } })
}

#[test]
fn initialize_schema() {
    // Should be able to initialize the database
    let connection = Connection::open_in_memory().unwrap();
    connection.execute_batch(SCHEMA).unwrap();
}

#[test]
fn initialise_canvas() {
    SqliteCanvas::new_in_memory().unwrap();
}

#[test]
fn add_layer() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();
    canvas.add_layer(CanvasLayerId::new(), None).unwrap();
}

#[test]
fn add_two_layers() {
    let mut canvas      = SqliteCanvas::new_in_memory().unwrap();
    let first_layer     = CanvasLayerId::new();
    let second_layer    = CanvasLayerId::new();

    canvas.add_layer(first_layer, None).unwrap();
    canvas.add_layer(second_layer, None).unwrap();

    let mut layers = vec![];
    canvas.query_document_outline(&mut layers).unwrap();
    assert!(layers == vec![
        VectorResponse::Document(vec![]),
        VectorResponse::Layer(first_layer, vec![]),
        VectorResponse::Layer(second_layer, vec![]),
        VectorResponse::LayerOrder(vec![first_layer, second_layer]),
    ], "{:?} ({:?} {:?})", layers, first_layer, second_layer);
}

#[test]
fn add_layer_before() {
    let mut canvas      = SqliteCanvas::new_in_memory().unwrap();
    let first_layer     = CanvasLayerId::new();
    let second_layer    = CanvasLayerId::new();

    canvas.add_layer(first_layer, None).unwrap();
    canvas.add_layer(second_layer, Some(first_layer)).unwrap();

    let mut layers = vec![];
    canvas.query_document_outline(&mut layers).unwrap();
    assert!(layers == vec![
        VectorResponse::Document(vec![]),
        VectorResponse::Layer(second_layer, vec![]),
        VectorResponse::Layer(first_layer, vec![]),
        VectorResponse::LayerOrder(vec![second_layer, first_layer]),
    ], "{:?} ({:?} {:?})", layers, first_layer, second_layer);
}

#[test]
fn set_property_ids() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();

    let property_1 = canvas.index_for_property(CanvasPropertyId::new("One")).unwrap();
    let property_2 = canvas.index_for_property(CanvasPropertyId::new("Two")).unwrap();

    assert!(property_1 == 1, "Property 1: {:?} != 1", property_1);
    assert!(property_2 == 2, "Property 2: {:?} != 2", property_2);
}

#[test]
fn read_property_ids_from_cache() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();

    // Write some properties
    canvas.index_for_property(CanvasPropertyId::new("One")).unwrap();
    canvas.index_for_property(CanvasPropertyId::new("Two")).unwrap();

    // Clear the cache
    canvas.property_id_cache.clear();

    // Re-fetch the properties
    let property_1 = canvas.index_for_property(CanvasPropertyId::new("One")).unwrap();
    let property_2 = canvas.index_for_property(CanvasPropertyId::new("Two")).unwrap();
    let property_3 = canvas.index_for_property(CanvasPropertyId::new("Three")).unwrap();

    assert!(property_1 == 1, "Property 1: {:?} != 1", property_1);
    assert!(property_2 == 2, "Property 2: {:?} != 2", property_2);
    assert!(property_3 == 3, "Property 3: {:?} != 3", property_3);
}

#[test]
fn set_document_properties() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();

    // Set some properties for the document
    canvas.set_document_properties(vec![
        (CanvasPropertyId::new("One"), CanvasProperty::Int(42)),
        (CanvasPropertyId::new("Two"), CanvasProperty::Float(42.0)),
        (CanvasPropertyId::new("Three"), CanvasProperty::IntList(vec![1, 2, 3])),
    ]).unwrap();
}

#[test]
fn set_layer_properties() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();

    // Create a layer
    let layer = CanvasLayerId::new();
    canvas.add_layer(layer, None).unwrap();

    // Set some properties for the layer
    canvas.set_layer_properties(layer, vec![
        (CanvasPropertyId::new("One"), CanvasProperty::Int(42)),
        (CanvasPropertyId::new("Two"), CanvasProperty::Float(42.0)),
        (CanvasPropertyId::new("Three"), CanvasProperty::IntList(vec![1, 2, 3])),
    ]).unwrap();
}

#[test]
fn remove_layer_deletes_properties() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer       = CanvasLayerId::new();

    canvas.add_layer(layer, None).unwrap();
    canvas.set_layer_properties(layer, vec![
        (CanvasPropertyId::new("Name"), CanvasProperty::Int(1)),
        (CanvasPropertyId::new("Visible"), CanvasProperty::Float(1.0)),
    ]).unwrap();

    let layer_idx = canvas.index_for_layer(layer).unwrap();

    // Verify properties exist
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerIntProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerFloatProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    assert!(int_count   > 0, "Layer should have int properties before removal");
    assert!(float_count > 0, "Layer should have float properties before removal");

    canvas.remove_layer(layer).unwrap();

    // Properties should be gone via CASCADE
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerIntProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerFloatProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 0, "Layer int properties should be deleted via CASCADE");
    assert!(float_count == 0, "Layer float properties should be deleted via CASCADE");
}

#[test]
fn add_shape() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();
    let shape = CanvasShapeId::new();

    canvas.add_shape(shape, test_rect()).unwrap();

    // Shape should exist in the database
    assert!(canvas.index_for_shape(shape).is_ok());
}

#[test]
fn add_shape_replaces_existing() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();
    let shape = CanvasShapeId::new();

    canvas.add_shape(shape, test_rect()).unwrap();
    let idx_before = canvas.index_for_shape(shape).unwrap();

    // Adding the same shape ID again should replace in place
    canvas.add_shape(shape, test_ellipse()).unwrap();
    let idx_after = canvas.index_for_shape(shape).unwrap();

    assert!(idx_before == idx_after, "ShapeId should be preserved on replace");

    // Verify the type was updated
    let shape_type: i64 = canvas.sqlite.query_one("SELECT ShapeType FROM Shapes WHERE ShapeId = ?", params![idx_after], |row| row.get(0)).unwrap();
    assert!(shape_type == CANVAS_ELLIPSE_V1_TYPE, "Shape type should be ellipse ({}), got {}", CANVAS_ELLIPSE_V1_TYPE, shape_type);
}

#[test]
fn remove_shape() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer       = CanvasLayerId::new();
    let shape       = CanvasShapeId::new();

    canvas.add_layer(layer, None).unwrap();
    canvas.add_shape(shape, test_rect()).unwrap();
    canvas.set_shape_parent(shape, CanvasShapeParent::Layer(layer)).unwrap();

    // Shape should be on the layer
    assert!(shapes_on_layer(&canvas, layer).len() == 1);

    canvas.remove_shape(shape).unwrap();

    // Shape should be gone from both Shapes and ShapeLayers
    assert!(canvas.index_for_shape(shape).is_err());
    assert!(shapes_on_layer(&canvas, layer).is_empty());
}

#[test]
fn remove_group_deletes_children() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let group       = CanvasShapeId::new();
    let child       = CanvasShapeId::new();

    canvas.add_shape(group, CanvasShape::Group).unwrap();
    canvas.add_shape(child, test_rect()).unwrap();
    canvas.set_shape_parent(child, CanvasShapeParent::Shape(group)).unwrap();
    assert!(shapes_in_group(&canvas, group).len() == 1);

    // Removing the group should also remove the child
    canvas.remove_shape(group).unwrap();
    assert!(canvas.index_for_shape(child).is_err(), "Child should be removed with group");
    assert!(canvas.index_for_shape(group).is_err(), "Group should be removed");
}

#[test]
fn remove_group_deletes_nested_children() {
    let mut canvas      = SqliteCanvas::new_in_memory().unwrap();
    let outer_group     = CanvasShapeId::new();
    let inner_group     = CanvasShapeId::new();
    let child           = CanvasShapeId::new();

    canvas.add_shape(outer_group, CanvasShape::Group).unwrap();
    canvas.add_shape(inner_group, CanvasShape::Group).unwrap();
    canvas.add_shape(child, test_rect()).unwrap();
    canvas.set_shape_parent(inner_group, CanvasShapeParent::Shape(outer_group)).unwrap();
    canvas.set_shape_parent(child, CanvasShapeParent::Shape(inner_group)).unwrap();

    // Removing the outer group should recursively remove inner group and child
    canvas.remove_shape(outer_group).unwrap();
    assert!(canvas.index_for_shape(outer_group).is_err(), "Outer group should be removed");
    assert!(canvas.index_for_shape(inner_group).is_err(), "Inner group should be removed");
    assert!(canvas.index_for_shape(child).is_err(), "Nested child should be removed");
}

#[test]
fn set_shape_definition() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();
    let shape = CanvasShapeId::new();

    canvas.add_shape(shape, test_rect()).unwrap();
    let shape_idx = canvas.index_for_shape(shape).unwrap();

    // Check initial type
    let shape_type: i64 = canvas.sqlite.query_one("SELECT ShapeType FROM Shapes WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    assert!(shape_type == CANVAS_RECTANGLE_V1_TYPE);

    // Replace definition with an ellipse
    canvas.set_shape_definition(shape, test_ellipse()).unwrap();
    let shape_type: i64 = canvas.sqlite.query_one("SELECT ShapeType FROM Shapes WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    assert!(shape_type == CANVAS_ELLIPSE_V1_TYPE);
}

#[test]
fn set_shape_parent_to_layer() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer       = CanvasLayerId::new();
    let shape       = CanvasShapeId::new();

    canvas.add_layer(layer, None).unwrap();
    canvas.add_shape(shape, test_rect()).unwrap();

    // Initially not on any layer
    assert!(shapes_on_layer(&canvas, layer).is_empty());

    canvas.set_shape_parent(shape, CanvasShapeParent::Layer(layer)).unwrap();
    assert!(shapes_on_layer(&canvas, layer) == vec![shape.to_string()]);
}

#[test]
fn set_shape_parent_to_group() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer       = CanvasLayerId::new();
    let group       = CanvasShapeId::new();
    let child       = CanvasShapeId::new();

    canvas.add_layer(layer, None).unwrap();
    canvas.add_shape(group, CanvasShape::Group).unwrap();
    canvas.add_shape(child, test_rect()).unwrap();

    // Put shape on a layer first, then move to group
    canvas.set_shape_parent(child, CanvasShapeParent::Layer(layer)).unwrap();
    assert!(shapes_on_layer(&canvas, layer) == vec![child.to_string()]);

    canvas.set_shape_parent(child, CanvasShapeParent::Shape(group)).unwrap();

    // Should be removed from the layer and added to the group
    assert!(shapes_on_layer(&canvas, layer).is_empty());
    assert!(shapes_in_group(&canvas, group) == vec![child.to_string()]);
}

#[test]
fn set_shape_parent_detach() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer       = CanvasLayerId::new();
    let shape       = CanvasShapeId::new();

    canvas.add_layer(layer, None).unwrap();
    canvas.add_shape(shape, test_rect()).unwrap();
    canvas.set_shape_parent(shape, CanvasShapeParent::Layer(layer)).unwrap();
    assert!(shapes_on_layer(&canvas, layer).len() == 1);

    canvas.set_shape_parent(shape, CanvasShapeParent::None).unwrap();
    assert!(shapes_on_layer(&canvas, layer).is_empty());
    assert!(canvas.index_for_shape(shape).is_ok(), "Shape should still exist after detach");
}

#[test]
fn reorder_shape_on_layer() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer       = CanvasLayerId::new();
    let shape_a     = CanvasShapeId::new();
    let shape_b     = CanvasShapeId::new();
    let shape_c     = CanvasShapeId::new();

    canvas.add_layer(layer, None).unwrap();
    canvas.add_shape(shape_a, test_rect()).unwrap();
    canvas.add_shape(shape_b, test_rect()).unwrap();
    canvas.add_shape(shape_c, test_rect()).unwrap();
    canvas.set_shape_parent(shape_a, CanvasShapeParent::Layer(layer)).unwrap();
    canvas.set_shape_parent(shape_b, CanvasShapeParent::Layer(layer)).unwrap();
    canvas.set_shape_parent(shape_c, CanvasShapeParent::Layer(layer)).unwrap();

    // Order is A, B, C
    assert!(shapes_on_layer(&canvas, layer) == vec![shape_a.to_string(), shape_b.to_string(), shape_c.to_string()]);

    // Move C before A -> C, A, B
    canvas.reorder_shape(shape_c, Some(shape_a)).unwrap();
    assert!(shapes_on_layer(&canvas, layer) == vec![shape_c.to_string(), shape_a.to_string(), shape_b.to_string()]);

    // Move A to end (before = None) -> C, B, A
    canvas.reorder_shape(shape_a, None).unwrap();
    assert!(shapes_on_layer(&canvas, layer) == vec![shape_c.to_string(), shape_b.to_string(), shape_a.to_string()]);
}

#[test]
fn reorder_shape_in_group() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let group       = CanvasShapeId::new();
    let shape_a     = CanvasShapeId::new();
    let shape_b     = CanvasShapeId::new();

    canvas.add_shape(group, CanvasShape::Group).unwrap();
    canvas.add_shape(shape_a, test_rect()).unwrap();
    canvas.add_shape(shape_b, test_rect()).unwrap();
    canvas.set_shape_parent(shape_a, CanvasShapeParent::Shape(group)).unwrap();
    canvas.set_shape_parent(shape_b, CanvasShapeParent::Shape(group)).unwrap();

    // Order is A, B
    assert!(shapes_in_group(&canvas, group) == vec![shape_a.to_string(), shape_b.to_string()]);

    // Move B before A -> B, A
    canvas.reorder_shape(shape_b, Some(shape_a)).unwrap();
    assert!(shapes_in_group(&canvas, group) == vec![shape_b.to_string(), shape_a.to_string()]);
}

#[test]
fn reorder_shape_different_parent_is_error() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer_1     = CanvasLayerId::new();
    let layer_2     = CanvasLayerId::new();
    let shape_a     = CanvasShapeId::new();
    let shape_b     = CanvasShapeId::new();

    canvas.add_layer(layer_1, None).unwrap();
    canvas.add_layer(layer_2, None).unwrap();
    canvas.add_shape(shape_a, test_rect()).unwrap();
    canvas.add_shape(shape_b, test_rect()).unwrap();
    canvas.set_shape_parent(shape_a, CanvasShapeParent::Layer(layer_1)).unwrap();
    canvas.set_shape_parent(shape_b, CanvasShapeParent::Layer(layer_2)).unwrap();

    // Trying to reorder shape_a before shape_b should fail because they have different parents
    let result = canvas.reorder_shape(shape_a, Some(shape_b));
    assert!(result.is_err(), "Reordering across different parents should fail (re-parent first)");

    // Shapes should be unchanged
    assert!(shapes_on_layer(&canvas, layer_1) == vec![shape_a.to_string()]);
    assert!(shapes_on_layer(&canvas, layer_2) == vec![shape_b.to_string()]);
}

#[test]
fn reorder_unparented_shape_is_error() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let shape       = CanvasShapeId::new();

    canvas.add_shape(shape, test_rect()).unwrap();

    // Shape has no parent, reorder should fail
    let result = canvas.reorder_shape(shape, None);
    assert!(result.is_err(), "Reordering a shape with no parent should fail");
}

#[test]
fn add_shape_brushes() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let shape       = CanvasShapeId::new();
    let brush_1     = CanvasBrushId::new();
    let brush_2     = CanvasBrushId::new();

    canvas.add_shape(shape, test_rect()).unwrap();
    insert_brush(&canvas, brush_1);
    insert_brush(&canvas, brush_2);

    canvas.add_shape_brushes(shape, vec![brush_1, brush_2]).unwrap();
    assert!(brushes_on_shape(&canvas, shape) == vec![brush_1.to_string(), brush_2.to_string()]);
}

#[test]
fn add_shape_brushes_appends() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let shape       = CanvasShapeId::new();
    let brush_1     = CanvasBrushId::new();
    let brush_2     = CanvasBrushId::new();
    let brush_3     = CanvasBrushId::new();

    canvas.add_shape(shape, test_rect()).unwrap();
    insert_brush(&canvas, brush_1);
    insert_brush(&canvas, brush_2);
    insert_brush(&canvas, brush_3);

    canvas.add_shape_brushes(shape, vec![brush_1]).unwrap();
    canvas.add_shape_brushes(shape, vec![brush_2, brush_3]).unwrap();
    assert!(brushes_on_shape(&canvas, shape) == vec![brush_1.to_string(), brush_2.to_string(), brush_3.to_string()]);
}

#[test]
fn remove_shape_brushes() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let shape       = CanvasShapeId::new();
    let brush_1     = CanvasBrushId::new();
    let brush_2     = CanvasBrushId::new();
    let brush_3     = CanvasBrushId::new();

    canvas.add_shape(shape, test_rect()).unwrap();
    insert_brush(&canvas, brush_1);
    insert_brush(&canvas, brush_2);
    insert_brush(&canvas, brush_3);

    canvas.add_shape_brushes(shape, vec![brush_1, brush_2, brush_3]).unwrap();
    canvas.remove_shape_brushes(shape, vec![brush_2]).unwrap();

    // brush_1 and brush_3 should remain in order
    assert!(brushes_on_shape(&canvas, shape) == vec![brush_1.to_string(), brush_3.to_string()]);
}

#[test]
fn remove_shape_deletes_properties() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let shape       = CanvasShapeId::new();

    canvas.add_shape(shape, test_rect()).unwrap();
    canvas.set_shape_properties(shape, vec![
        (CanvasPropertyId::new("Color"), CanvasProperty::Int(255)),
        (CanvasPropertyId::new("Width"), CanvasProperty::Float(1.5)),
    ]).unwrap();

    let shape_idx = canvas.index_for_shape(shape).unwrap();

    // Verify properties exist
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeIntProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeFloatProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    assert!(int_count   > 0, "Shape should have int properties before removal");
    assert!(float_count > 0, "Shape should have float properties before removal");

    canvas.remove_shape(shape).unwrap();

    // Properties should be gone via CASCADE
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeIntProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeFloatProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 0, "Shape int properties should be deleted via CASCADE");
    assert!(float_count == 0, "Shape float properties should be deleted via CASCADE");
}

#[test]
fn remove_brush_deletes_properties() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let brush       = CanvasBrushId::new();

    canvas.add_brush(brush).unwrap();
    canvas.set_brush_properties(brush, vec![
        (CanvasPropertyId::new("Size"), CanvasProperty::Int(10)),
        (CanvasPropertyId::new("Opacity"), CanvasProperty::Float(0.5)),
    ]).unwrap();

    let brush_idx = canvas.index_for_brush(brush).unwrap();

    // Verify properties exist
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushIntProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushFloatProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    assert!(int_count   > 0, "Brush should have int properties before removal");
    assert!(float_count > 0, "Brush should have float properties before removal");

    canvas.remove_brush(brush).unwrap();

    // Properties should be gone via CASCADE
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushIntProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushFloatProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 0, "Brush int properties should be deleted via CASCADE");
    assert!(float_count == 0, "Brush float properties should be deleted via CASCADE");
}

#[test]
fn delete_document_properties() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();

    // Set some document properties of different types
    canvas.set_properties(CanvasPropertyTarget::Document, vec![
        (CanvasPropertyId::new("Name"), CanvasProperty::Int(1)),
        (CanvasPropertyId::new("Version"), CanvasProperty::Float(2.0)),
        (CanvasPropertyId::new("Data"), CanvasProperty::ByteList(vec![1, 2, 3])),
    ]).unwrap();

    // Verify properties exist
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentIntProperties", params![], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentFloatProperties", params![], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentBlobProperties", params![], |row| row.get(0)).unwrap();
    assert!(int_count   == 1, "Document should have 1 int property, got {}", int_count);
    assert!(float_count == 1, "Document should have 1 float property, got {}", float_count);
    assert!(blob_count  == 1, "Document should have 1 blob property, got {}", blob_count);

    // Delete the int and float properties
    canvas.delete_properties(CanvasPropertyTarget::Document, vec![
        CanvasPropertyId::new("Name"),
        CanvasPropertyId::new("Version"),
    ]).unwrap();

    // Int and float properties should be gone, blob should remain
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentIntProperties", params![], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentFloatProperties", params![], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentBlobProperties", params![], |row| row.get(0)).unwrap();
    assert!(int_count   == 0, "Document int property should be deleted, got {}", int_count);
    assert!(float_count == 0, "Document float property should be deleted, got {}", float_count);
    assert!(blob_count  == 1, "Document blob property should still exist, got {}", blob_count);
}

#[test]
fn delete_document_properties_all_types() {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();

    // Set properties covering all types
    canvas.set_properties(CanvasPropertyTarget::Document, vec![
        (CanvasPropertyId::new("IntProp"), CanvasProperty::Int(42)),
        (CanvasPropertyId::new("FloatProp"), CanvasProperty::Float(3.14)),
        (CanvasPropertyId::new("BlobProp"), CanvasProperty::ByteList(vec![10, 20])),
    ]).unwrap();

    // Delete all three properties at once
    canvas.delete_properties(CanvasPropertyTarget::Document, vec![
        CanvasPropertyId::new("IntProp"),
        CanvasPropertyId::new("FloatProp"),
        CanvasPropertyId::new("BlobProp"),
    ]).unwrap();

    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentIntProperties", params![], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentFloatProperties", params![], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM DocumentBlobProperties", params![], |row| row.get(0)).unwrap();
    assert!(int_count   == 0, "All document int properties should be deleted");
    assert!(float_count == 0, "All document float properties should be deleted");
    assert!(blob_count  == 0, "All document blob properties should be deleted");
}

#[test]
fn delete_shape_properties() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let shape       = CanvasShapeId::new();

    canvas.add_shape(shape, test_rect()).unwrap();
    canvas.set_properties(CanvasPropertyTarget::Shape(shape), vec![
        (CanvasPropertyId::new("Color"), CanvasProperty::Int(255)),
        (CanvasPropertyId::new("Width"), CanvasProperty::Float(1.5)),
        (CanvasPropertyId::new("Points"), CanvasProperty::FloatList(vec![0.0, 1.0, 2.0])),
    ]).unwrap();

    let shape_idx = canvas.index_for_shape(shape).unwrap();

    // Verify properties exist
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeIntProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeFloatProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeBlobProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 1, "Shape should have 1 int property");
    assert!(float_count == 1, "Shape should have 1 float property");
    assert!(blob_count  == 1, "Shape should have 1 blob property");

    // Delete just the int property
    canvas.delete_properties(CanvasPropertyTarget::Shape(shape), vec![
        CanvasPropertyId::new("Color"),
    ]).unwrap();

    // Only the int property should be gone
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeIntProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeFloatProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeBlobProperties WHERE ShapeId = ?", params![shape_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 0, "Shape int property should be deleted");
    assert!(float_count == 1, "Shape float property should still exist");
    assert!(blob_count  == 1, "Shape blob property should still exist");
}

#[test]
fn delete_shape_properties_does_not_affect_other_shapes() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let shape_a     = CanvasShapeId::new();
    let shape_b     = CanvasShapeId::new();

    canvas.add_shape(shape_a, test_rect()).unwrap();
    canvas.add_shape(shape_b, test_rect()).unwrap();

    // Set the same property on both shapes
    canvas.set_properties(CanvasPropertyTarget::Shape(shape_a), vec![
        (CanvasPropertyId::new("Color"), CanvasProperty::Int(100)),
    ]).unwrap();
    canvas.set_properties(CanvasPropertyTarget::Shape(shape_b), vec![
        (CanvasPropertyId::new("Color"), CanvasProperty::Int(200)),
    ]).unwrap();

    // Delete the property from shape_a only
    canvas.delete_properties(CanvasPropertyTarget::Shape(shape_a), vec![
        CanvasPropertyId::new("Color"),
    ]).unwrap();

    let shape_a_idx = canvas.index_for_shape(shape_a).unwrap();
    let shape_b_idx = canvas.index_for_shape(shape_b).unwrap();

    let a_count: i64 = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeIntProperties WHERE ShapeId = ?", params![shape_a_idx], |row| row.get(0)).unwrap();
    let b_count: i64 = canvas.sqlite.query_one("SELECT COUNT(*) FROM ShapeIntProperties WHERE ShapeId = ?", params![shape_b_idx], |row| row.get(0)).unwrap();
    assert!(a_count == 0, "Shape A int property should be deleted");
    assert!(b_count == 1, "Shape B int property should be unaffected");
}

#[test]
fn delete_brush_properties() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let brush       = CanvasBrushId::new();

    canvas.add_brush(brush).unwrap();
    canvas.set_properties(CanvasPropertyTarget::Brush(brush), vec![
        (CanvasPropertyId::new("Size"), CanvasProperty::Int(10)),
        (CanvasPropertyId::new("Opacity"), CanvasProperty::Float(0.5)),
        (CanvasPropertyId::new("Pattern"), CanvasProperty::ByteList(vec![0, 1, 0, 1])),
    ]).unwrap();

    let brush_idx = canvas.index_for_brush(brush).unwrap();

    // Verify properties exist
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushIntProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushFloatProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushBlobProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 1, "Brush should have 1 int property");
    assert!(float_count == 1, "Brush should have 1 float property");
    assert!(blob_count  == 1, "Brush should have 1 blob property");

    // Delete the float and blob properties, leaving int
    canvas.delete_properties(CanvasPropertyTarget::Brush(brush), vec![
        CanvasPropertyId::new("Opacity"),
        CanvasPropertyId::new("Pattern"),
    ]).unwrap();

    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushIntProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushFloatProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM BrushBlobProperties WHERE BrushId = ?", params![brush_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 1, "Brush int property should still exist");
    assert!(float_count == 0, "Brush float property should be deleted");
    assert!(blob_count  == 0, "Brush blob property should be deleted");
}

#[test]
fn delete_layer_properties() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer       = CanvasLayerId::new();

    canvas.add_layer(layer, None).unwrap();
    canvas.set_properties(CanvasPropertyTarget::Layer(layer), vec![
        (CanvasPropertyId::new("Name"), CanvasProperty::Int(1)),
        (CanvasPropertyId::new("Visible"), CanvasProperty::Float(1.0)),
        (CanvasPropertyId::new("Metadata"), CanvasProperty::IntList(vec![10, 20, 30])),
    ]).unwrap();

    let layer_idx = canvas.index_for_layer(layer).unwrap();

    // Verify properties exist
    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerIntProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerFloatProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerBlobProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 1, "Layer should have 1 int property");
    assert!(float_count == 1, "Layer should have 1 float property");
    assert!(blob_count  == 1, "Layer should have 1 blob property");

    // Delete just the float property
    canvas.delete_properties(CanvasPropertyTarget::Layer(layer), vec![
        CanvasPropertyId::new("Visible"),
    ]).unwrap();

    let int_count: i64      = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerIntProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    let float_count: i64    = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerFloatProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    let blob_count: i64     = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerBlobProperties WHERE LayerId = ?", params![layer_idx], |row| row.get(0)).unwrap();
    assert!(int_count   == 1, "Layer int property should still exist");
    assert!(float_count == 0, "Layer float property should be deleted");
    assert!(blob_count  == 1, "Layer blob property should still exist");
}

#[test]
fn delete_layer_properties_does_not_affect_other_layers() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let layer_a     = CanvasLayerId::new();
    let layer_b     = CanvasLayerId::new();

    canvas.add_layer(layer_a, None).unwrap();
    canvas.add_layer(layer_b, None).unwrap();

    // Set the same property on both layers
    canvas.set_properties(CanvasPropertyTarget::Layer(layer_a), vec![
        (CanvasPropertyId::new("Name"), CanvasProperty::Int(1)),
    ]).unwrap();
    canvas.set_properties(CanvasPropertyTarget::Layer(layer_b), vec![
        (CanvasPropertyId::new("Name"), CanvasProperty::Int(2)),
    ]).unwrap();

    // Delete the property from layer_a only
    canvas.delete_properties(CanvasPropertyTarget::Layer(layer_a), vec![
        CanvasPropertyId::new("Name"),
    ]).unwrap();

    let layer_a_idx = canvas.index_for_layer(layer_a).unwrap();
    let layer_b_idx = canvas.index_for_layer(layer_b).unwrap();

    let a_count: i64 = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerIntProperties WHERE LayerId = ?", params![layer_a_idx], |row| row.get(0)).unwrap();
    let b_count: i64 = canvas.sqlite.query_one("SELECT COUNT(*) FROM LayerIntProperties WHERE LayerId = ?", params![layer_b_idx], |row| row.get(0)).unwrap();
    assert!(a_count == 0, "Layer A int property should be deleted");
    assert!(b_count == 1, "Layer B int property should be unaffected");
}

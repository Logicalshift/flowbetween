use super::*;
use super::super::point::*;

use super::super::brush::*;
use super::super::layer::*;
use super::super::property::*;
use super::super::queries::*;
use super::super::shape::*;
use super::super::vector_editor::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_scene::commands::*;

use futures::prelude::*;
use rusqlite::*;
use ::serde::*;

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
fn query_document_outline() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct TestResponse(Vec<VectorResponse>);

    impl SceneMessage for TestResponse { }

    let test_program    = SubProgramId::new();
    let query_program   = SubProgramId::new();

    let layer_1         = CanvasLayerId::new();
    let layer_2         = CanvasLayerId::new();

    // Program that adds some layers and sends a test response
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let _sqlite     = context.send::<SqliteCanvasRequest>(()).unwrap();
        let mut canvas  = context.send(()).unwrap();

        // Set up some layers (layer2 vs layer1)
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_1, before_layer: None }).await.unwrap();
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_2, before_layer: Some(layer_1) }).await.unwrap();

        // Query the document outline
        let outline = context.spawn_query(ReadCommand::default(), VectorQuery::DocumentOutline(().into()), ()).unwrap();
        let outline = outline.collect::<Vec<_>>().await;

        context.send_message(TestResponse(outline)).await.unwrap();
    }, 1);

    // The expected response to the query after this set up
    let expected = vec![
        VectorResponse::Document(vec![]),
        VectorResponse::Layer(layer_2, vec![]), 
        VectorResponse::Layer(layer_1, vec![]),
        VectorResponse::LayerOrder(vec![layer_2, layer_1]), 
    ];

    // Run the test
    TestBuilder::new()
        .expect_message_matching(TestResponse(expected), format!("Layer 1 = {:?}, layer 2 = {:?}", layer_1, layer_2))
        .run_in_scene(&scene, test_program);
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
fn remove_group_detaches_children() {
    let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
    let group       = CanvasShapeId::new();
    let child       = CanvasShapeId::new();

    canvas.add_shape(group, CanvasShape::Group).unwrap();
    canvas.add_shape(child, test_rect()).unwrap();
    canvas.set_shape_parent(child, CanvasShapeParent::Shape(group)).unwrap();
    assert!(shapes_in_group(&canvas, group).len() == 1);

    // Removing the group should detach the child
    canvas.remove_shape(group).unwrap();
    assert!(canvas.index_for_shape(child).is_ok(), "Child should still exist");
    assert!(canvas.index_for_shape(group).is_err(), "Group should be removed");

    // Child should no longer be in any group
    let child_idx       = canvas.index_for_shape(child).unwrap();
    let in_any_group    = canvas.sqlite.query_one::<i64, _, _>("SELECT COUNT(*) FROM ShapeGroups WHERE ShapeId = ?", params![child_idx], |row| row.get(0)).unwrap();
    assert!(in_any_group == 0, "Child should not be in any group after parent is removed");
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

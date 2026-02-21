use super::*;
use super::super::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_scene::commands::*;

use futures::prelude::*;
use ::serde::*;
use std::time::{Duration};

fn test_rect() -> CanvasShape {
    CanvasShape::Rectangle(CanvasRectangle { min: CanvasPoint { x: 0.0, y: 0.0 }, max: CanvasPoint { x: 10.0, y: 10.0 } })
}

fn test_ellipse() -> CanvasShape {
    CanvasShape::Ellipse(CanvasEllipse { min: CanvasPoint { x: 0.0, y: 0.0 }, max: CanvasPoint { x: 5.0, y: 5.0 }, direction: CanvasPoint { x: 1.0, y: 0.0 } })
}

fn test_shape_type() -> ShapeType {
    ShapeType::new("flowbetween::test")
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

        // Set some properties
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Document, vec![(CanvasPropertyId::new("test::document::property"), CanvasProperty::Int(42))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Layer(layer_2), vec![(CanvasPropertyId::new("test::layer::property"), CanvasProperty::Int(43))])).await.unwrap();

        // Query the document outline
        let outline = context.spawn_query(ReadCommand::default(), VectorQuery::DocumentOutline(().into()), ()).unwrap();
        let outline = outline.collect::<Vec<_>>().await;

        context.send_message(TestResponse(outline)).await.unwrap();
    }, 1);

    // The expected response to the query after this set up
    let expected = vec![
        VectorResponse::Document(vec![(CanvasPropertyId::new("test::document::property"), CanvasProperty::Int(42))]),
        VectorResponse::Layer(layer_2, vec![(CanvasPropertyId::new("test::layer::property"), CanvasProperty::Int(43))]),
        VectorResponse::Layer(layer_1, vec![]),
        VectorResponse::LayerOrder(vec![layer_2, layer_1]),
    ];

    // Run the test
    TestBuilder::new()
        .expect_message_matching(TestResponse(expected), format!("Layer 1 = {:?}, layer 2 = {:?}", layer_1, layer_2))
        .run_in_scene(&scene, test_program);
}

#[test]
fn query_shape_properties() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct TestResponse(Vec<VectorResponse>);

    impl SceneMessage for TestResponse { }

    let test_program    = SubProgramId::new();
    let query_program   = SubProgramId::new();

    let shape_1         = CanvasShapeId::new();
    let shape_2         = CanvasShapeId::new();
    let brush_1         = CanvasBrushId::new();
    let brush_2         = CanvasBrushId::new();

    // Program that adds some layers and sends a test response
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let _sqlite     = context.send::<SqliteCanvasRequest>(()).unwrap();
        let mut canvas  = context.send(()).unwrap();

        // Set up some shapes with properties
        canvas.send(VectorCanvas::AddShape(shape_1, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();
        canvas.send(VectorCanvas::AddShape(shape_2, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();

        canvas.send(VectorCanvas::AddBrush(brush_1)).await.unwrap();
        canvas.send(VectorCanvas::AddBrush(brush_2)).await.unwrap();
        canvas.send(VectorCanvas::AddShapeBrushes(shape_2, vec![brush_1, brush_2])).await.unwrap();

        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_1), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(42))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_2), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(43))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Brush(brush_1), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(44)), (CanvasPropertyId::new("brush1"), CanvasProperty::Int(45)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(46))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Brush(brush_2), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(47)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(49))])).await.unwrap();

        // Query the document outline
        let outline = context.spawn_query(ReadCommand::default(), VectorQuery::Shapes(().into(), vec![shape_1, shape_2]), ()).unwrap();
        let outline = outline.collect::<Vec<_>>().await;

        context.send_message(TestResponse(outline)).await.unwrap();
    }, 1);

    // The expected response to the query after this set up
    let expected = vec![
        VectorResponse::Shape(shape_1, CanvasShape::Group, ShapeType::new("shape"), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(42))]),
        VectorResponse::Shape(shape_2, CanvasShape::Group, ShapeType::new("shape"), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(43)), (CanvasPropertyId::new("brush1"), CanvasProperty::Int(45)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(49))]),
    ];

    // Run the test
    TestBuilder::new()
        .expect_message_matching(TestResponse(expected), "")
        .run_in_scene(&scene, test_program);
}

#[test]
fn query_brush_properties() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct TestResponse(Vec<VectorResponse>);

    impl SceneMessage for TestResponse { }

    let test_program    = SubProgramId::new();
    let query_program   = SubProgramId::new();

    let shape_1         = CanvasShapeId::new();
    let shape_2         = CanvasShapeId::new();
    let brush_1         = CanvasBrushId::new();
    let brush_2         = CanvasBrushId::new();

    // Program that adds some layers and sends a test response
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let _sqlite     = context.send::<SqliteCanvasRequest>(()).unwrap();
        let mut canvas  = context.send(()).unwrap();

        // Set up some shapes with properties
        canvas.send(VectorCanvas::AddShape(shape_1, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();
        canvas.send(VectorCanvas::AddShape(shape_2, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();

        canvas.send(VectorCanvas::AddBrush(brush_1)).await.unwrap();
        canvas.send(VectorCanvas::AddBrush(brush_2)).await.unwrap();
        canvas.send(VectorCanvas::AddShapeBrushes(shape_2, vec![brush_1, brush_2])).await.unwrap();

        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_1), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(42))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_2), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(43))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Brush(brush_1), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(44)), (CanvasPropertyId::new("brush1"), CanvasProperty::Int(45)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(46))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Brush(brush_2), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(47)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(49))])).await.unwrap();

        // Query the document outline
        let outline = context.spawn_query(ReadCommand::default(), VectorQuery::Brushes(().into(), vec![brush_1, brush_2]), ()).unwrap();
        let outline = outline.collect::<Vec<_>>().await;

        context.send_message(TestResponse(outline)).await.unwrap();
    }, 1);

    // The expected response to the query after this set up
    let expected = vec![
        VectorResponse::Brush(brush_1, vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(44)), (CanvasPropertyId::new("brush1"), CanvasProperty::Int(45)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(46))]),
        VectorResponse::Brush(brush_2, vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(47)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(49))]),
    ];

    // Run the test
    TestBuilder::new()
        .expect_message_matching(TestResponse(expected), "")
        .run_in_scene(&scene, test_program);
}

#[test]
fn query_layer() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct TestResponse(Vec<VectorResponse>);

    impl SceneMessage for TestResponse { }

    let test_program    = SubProgramId::new();
    let query_program   = SubProgramId::new();

    let layer_1         = CanvasLayerId::new();
    let layer_2         = CanvasLayerId::new();
    let shape_1         = CanvasShapeId::new();
    let shape_2         = CanvasShapeId::new();
    let shape_3         = CanvasShapeId::new();
    let shape_4         = CanvasShapeId::new();
    let brush_1         = CanvasBrushId::new();
    let brush_2         = CanvasBrushId::new();

    // Program that adds some layers and sends a test response
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let _sqlite     = context.send::<SqliteCanvasRequest>(()).unwrap();
        let mut canvas  = context.send(()).unwrap();

        // Set up some layers with shapes on them. Shape 1 & 2 are on layer 1 and shape 3 & 4 are on layer 2
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_1, before_layer: None }).await.unwrap();
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_2, before_layer: None }).await.unwrap();

        canvas.send(VectorCanvas::AddShape(shape_1, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();
        canvas.send(VectorCanvas::AddShape(shape_2, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();
        canvas.send(VectorCanvas::AddShape(shape_3, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();
        canvas.send(VectorCanvas::AddShape(shape_4, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();

        canvas.send(VectorCanvas::AddBrush(brush_1)).await.unwrap();
        canvas.send(VectorCanvas::AddBrush(brush_2)).await.unwrap();
        canvas.send(VectorCanvas::AddShapeBrushes(shape_2, vec![brush_1, brush_2])).await.unwrap();

        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_1), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(42))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_2), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(43))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Brush(brush_1), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(44)), (CanvasPropertyId::new("brush1"), CanvasProperty::Int(45)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(46))])).await.unwrap();
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Brush(brush_2), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(47)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(49))])).await.unwrap();

        canvas.send(VectorCanvas::SetShapeParent(shape_1, CanvasShapeParent::Layer(layer_1, Duration::from_nanos(0)))).await.ok();
        canvas.send(VectorCanvas::SetShapeParent(shape_2, CanvasShapeParent::Layer(layer_1, Duration::from_nanos(0)))).await.ok();
        canvas.send(VectorCanvas::SetShapeParent(shape_3, CanvasShapeParent::Layer(layer_2, Duration::from_nanos(0)))).await.ok();
        canvas.send(VectorCanvas::SetShapeParent(shape_4, CanvasShapeParent::Layer(layer_2, Duration::from_nanos(0)))).await.ok();

        // Query the layers in a few ways (we start with the simpler second layer)
        let second_layer = context.spawn_query(ReadCommand::default(), VectorQuery::Layers(().into(), vec![layer_2], Duration::ZERO), ()).unwrap();
        let second_layer = second_layer.collect::<Vec<_>>().await;

        context.send_message(TestResponse(second_layer)).await.unwrap();

        let first_layer = context.spawn_query(ReadCommand::default(), VectorQuery::Layers(().into(), vec![layer_1], Duration::ZERO), ()).unwrap();
        let first_layer = first_layer.collect::<Vec<_>>().await;

        context.send_message(TestResponse(first_layer)).await.unwrap();

        let all_layers = context.spawn_query(ReadCommand::default(), VectorQuery::Layers(().into(), vec![layer_1, layer_2], Duration::ZERO), ()).unwrap();
        let all_layers = all_layers.collect::<Vec<_>>().await;

        context.send_message(TestResponse(all_layers)).await.unwrap();
    }, 1);

    // The expected response to the query after this set up
    let expected_first = vec![
        VectorResponse::Layer(layer_1, vec![]),
        VectorResponse::Shape(shape_1, CanvasShape::Group, ShapeType::new("shape"), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(42))]),
        VectorResponse::Shape(shape_2, CanvasShape::Group, ShapeType::new("shape"), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(43)), (CanvasPropertyId::new("brush1"), CanvasProperty::Int(45)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(49))]),
    ];
    let expected_second = vec![
        VectorResponse::Layer(layer_2, vec![]),
        VectorResponse::Shape(shape_3, CanvasShape::Group, ShapeType::new("shape"), vec![]),
        VectorResponse::Shape(shape_4, CanvasShape::Group, ShapeType::new("shape"), vec![]),
    ];
    let expected_all = vec![
        VectorResponse::Layer(layer_1, vec![]),
        VectorResponse::Shape(shape_1, CanvasShape::Group, ShapeType::new("shape"), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(42))]),
        VectorResponse::Shape(shape_2, CanvasShape::Group, ShapeType::new("shape"), vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(43)), (CanvasPropertyId::new("brush1"), CanvasProperty::Int(45)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(49))]),

        VectorResponse::Layer(layer_2, vec![]),
        VectorResponse::Shape(shape_3, CanvasShape::Group, ShapeType::new("shape"), vec![]),
        VectorResponse::Shape(shape_4, CanvasShape::Group, ShapeType::new("shape"), vec![]),
    ];

    // Run the test
    TestBuilder::new()
        .expect_message_matching(TestResponse(expected_second), "Second layer failed (no properties)")
        .expect_message_matching(TestResponse(expected_first), "First layer failed (properties)")
        .expect_message_matching(TestResponse(expected_all), "All layers failed")
        .run_in_scene(&scene, test_program);
}

#[test]
fn query_layer_with_groups() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct TestResponse(Vec<VectorResponse>);

    impl SceneMessage for TestResponse { }

    let test_program    = SubProgramId::new();
    let query_program   = SubProgramId::new();

    let layer           = CanvasLayerId::new();
    let group_shape     = CanvasShapeId::new();
    let child_1         = CanvasShapeId::new();
    let nested_group    = CanvasShapeId::new();
    let nested_child_1  = CanvasShapeId::new();
    let nested_child_2  = CanvasShapeId::new();
    let nested2_child_1 = CanvasShapeId::new();
    let nested2_child_2 = CanvasShapeId::new();
    let child_2         = CanvasShapeId::new();
    let after_group     = CanvasShapeId::new();

    // Program that sets up a layer with nested groups
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let _sqlite     = context.send::<SqliteCanvasRequest>(()).unwrap();
        let mut canvas  = context.send(()).unwrap();

        // Create the layer and shapes
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer, before_layer: None }).await.unwrap();

        canvas.send(VectorCanvas::AddShape(group_shape, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();
        canvas.send(VectorCanvas::AddShape(child_1, ShapeType::new("shape"), test_rect())).await.unwrap();
        canvas.send(VectorCanvas::AddShape(nested_group, ShapeType::new("shape"), CanvasShape::Group)).await.unwrap();
        canvas.send(VectorCanvas::AddShape(nested_child_1, ShapeType::new("shape"), test_ellipse())).await.unwrap();
        canvas.send(VectorCanvas::AddShape(nested_child_2, ShapeType::new("shape"), test_rect())).await.unwrap();
        canvas.send(VectorCanvas::AddShape(nested2_child_1, ShapeType::new("shape"), test_ellipse())).await.unwrap();
        canvas.send(VectorCanvas::AddShape(nested2_child_2, ShapeType::new("shape"), test_rect())).await.unwrap();
        canvas.send(VectorCanvas::AddShape(child_2, ShapeType::new("shape"), test_ellipse())).await.unwrap();
        canvas.send(VectorCanvas::AddShape(after_group, ShapeType::new("shape"), test_rect())).await.unwrap();

        // Parent the group shape to the layer
        canvas.send(VectorCanvas::SetShapeParent(group_shape, CanvasShapeParent::Layer(layer, Duration::from_nanos(0)))).await.ok();

        // Parent children to the group shape
        canvas.send(VectorCanvas::SetShapeParent(child_1, CanvasShapeParent::Shape(group_shape))).await.ok();
        canvas.send(VectorCanvas::SetShapeParent(nested_group, CanvasShapeParent::Shape(group_shape))).await.ok();
        canvas.send(VectorCanvas::SetShapeParent(child_2, CanvasShapeParent::Shape(group_shape))).await.ok();

        // Parent nested children to the nested group
        canvas.send(VectorCanvas::SetShapeParent(nested_child_1, CanvasShapeParent::Shape(nested_group))).await.ok();
        canvas.send(VectorCanvas::SetShapeParent(nested_child_2, CanvasShapeParent::Shape(nested_group))).await.ok();
        canvas.send(VectorCanvas::SetShapeParent(nested2_child_1, CanvasShapeParent::Shape(nested_child_2))).await.ok();
        canvas.send(VectorCanvas::SetShapeParent(nested2_child_2, CanvasShapeParent::Shape(nested_child_2))).await.ok();

        // Parent the trailing shape to the layer
        canvas.send(VectorCanvas::SetShapeParent(after_group, CanvasShapeParent::Layer(layer, Duration::from_nanos(0)))).await.ok();

        // Query the layer
        let layer_result = context.spawn_query(ReadCommand::default(), VectorQuery::Layers(().into(), vec![layer], Duration::ZERO), ()).unwrap();
        let layer_result = layer_result.collect::<Vec<_>>().await;

        context.send_message(TestResponse(layer_result)).await.unwrap();
    }, 1);

    // The expected response, with two nested groups
    let expected = vec![
        VectorResponse::Layer(layer, vec![]),
        VectorResponse::Shape(group_shape, CanvasShape::Group, ShapeType::new("shape"), vec![]),
        VectorResponse::StartGroup,
        VectorResponse::Shape(child_1, test_rect(), ShapeType::new("shape"), vec![]),
        VectorResponse::Shape(nested_group, CanvasShape::Group, ShapeType::new("shape"), vec![]),
        VectorResponse::StartGroup,
        VectorResponse::Shape(nested_child_1, test_ellipse(), ShapeType::new("shape"), vec![]),
        VectorResponse::Shape(nested_child_2, test_rect(), ShapeType::new("shape"), vec![]),
        VectorResponse::StartGroup,
        VectorResponse::Shape(nested2_child_1, test_ellipse(), ShapeType::new("shape"), vec![]),
        VectorResponse::Shape(nested2_child_2, test_rect(), ShapeType::new("shape"), vec![]),
        VectorResponse::EndGroup,
        VectorResponse::EndGroup,
        VectorResponse::Shape(child_2, test_ellipse(), ShapeType::new("shape"), vec![]),
        VectorResponse::EndGroup,
        VectorResponse::Shape(after_group, test_rect(), ShapeType::new("shape"), vec![]),
    ];

    // Run the test
    TestBuilder::new()
        .expect_message_matching(TestResponse(expected), "Layer with nested groups")
        .run_in_scene(&scene, test_program);
}

#[test]
fn subscribe_to_canvas_updates() {
    let scene = Scene::default();

    let test_program    = SubProgramId::new();
    let editor_program  = SubProgramId::new();

    let layer_1     = CanvasLayerId::new();
    let layer_2     = CanvasLayerId::new();
    let shape_1     = CanvasShapeId::new();
    let shape_2     = CanvasShapeId::new();
    let group       = CanvasShapeId::new();
    let brush_1     = CanvasBrushId::new();

    // Program that subscribes the test program and then sends edits one at a time
    scene.add_subprogram(editor_program, move |_input: InputStream<()>, context| async move {
        let _sqlite     = context.send::<SqliteCanvasRequest>(()).unwrap();
        let mut canvas  = context.send(()).unwrap();

        // Subscribe the test program to canvas updates
        canvas.send(VectorCanvas::Subscribe(test_program.into())).await.unwrap();
        context.wait_for_idle(100).await;

        // AddLayer → LayerChanged
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_1, before_layer: None }).await.unwrap();
        context.wait_for_idle(100).await;

        // AddLayer (second layer for later use) → LayerChanged
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_2, before_layer: None }).await.unwrap();
        context.wait_for_idle(100).await;

        // ReorderLayer → LayerChanged
        canvas.send(VectorCanvas::ReorderLayer { layer_id: layer_2, before_layer: Some(layer_1) }).await.unwrap();
        context.wait_for_idle(100).await;

        // AddShape → ShapeChanged
        canvas.send(VectorCanvas::AddShape(shape_1, test_shape_type(), test_rect())).await.unwrap();
        context.wait_for_idle(100).await;

        // SetShapeParent to layer → LayerChanged + ShapeChanged
        canvas.send(VectorCanvas::SetShapeParent(shape_1, CanvasShapeParent::Layer(layer_1, Duration::from_nanos(0)))).await.unwrap();
        context.wait_for_idle(100).await;

        // SetShapeDefinition → ShapeChanged
        canvas.send(VectorCanvas::SetShapeDefinition(shape_1, test_ellipse())).await.unwrap();
        context.wait_for_idle(100).await;

        // AddBrush → no update
        canvas.send(VectorCanvas::AddBrush(brush_1)).await.unwrap();
        context.wait_for_idle(100).await;

        // AddShapeBrushes → ShapeChanged
        canvas.send(VectorCanvas::AddShapeBrushes(shape_1, vec![brush_1])).await.unwrap();
        context.wait_for_idle(100).await;

        // SetProperty on brush → ShapeChanged (notifies shapes the brush is attached to)
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Brush(brush_1), vec![(CanvasPropertyId::new("Size"), CanvasProperty::Int(10))])).await.unwrap();
        context.wait_for_idle(100).await;

        // RemoveProperty on brush → ShapeChanged (notifies shapes the brush is attached to)
        canvas.send(VectorCanvas::RemoveProperty(CanvasPropertyTarget::Brush(brush_1), vec![CanvasPropertyId::new("Size")])).await.unwrap();
        context.wait_for_idle(100).await;

        // RemoveShapeBrushes → ShapeChanged
        canvas.send(VectorCanvas::RemoveShapeBrushes(shape_1, vec![brush_1])).await.unwrap();
        context.wait_for_idle(100).await;

        // SetProperty on layer → LayerChanged
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Layer(layer_1), vec![(CanvasPropertyId::new("Name"), CanvasProperty::Int(1))])).await.unwrap();
        context.wait_for_idle(100).await;

        // SetProperty on shape → ShapeChanged
        canvas.send(VectorCanvas::SetProperty(CanvasPropertyTarget::Shape(shape_1), vec![(CanvasPropertyId::new("Color"), CanvasProperty::Int(255))])).await.unwrap();
        context.wait_for_idle(100).await;

        // RemoveProperty on shape → ShapeChanged
        canvas.send(VectorCanvas::RemoveProperty(CanvasPropertyTarget::Shape(shape_1), vec![CanvasPropertyId::new("Color")])).await.unwrap();
        context.wait_for_idle(100).await;

        // RemoveProperty on layer → LayerChanged
        canvas.send(VectorCanvas::RemoveProperty(CanvasPropertyTarget::Layer(layer_1), vec![CanvasPropertyId::new("Name")])).await.unwrap();
        context.wait_for_idle(100).await;

        // Set up shape_2 on layer_1 for ReorderShape
        canvas.send(VectorCanvas::AddShape(shape_2, test_shape_type(), test_rect())).await.unwrap();
        context.wait_for_idle(100).await;
        canvas.send(VectorCanvas::SetShapeParent(shape_2, CanvasShapeParent::Layer(layer_1, Duration::from_nanos(0)))).await.unwrap();
        context.wait_for_idle(100).await;

        // ReorderShape → ShapeChanged
        canvas.send(VectorCanvas::ReorderShape { shape_id: shape_2, before_shape: Some(shape_1) }).await.unwrap();
        context.wait_for_idle(100).await;

        // SetShapeParent to group → ShapeChanged (contains both the shape and the parent)
        canvas.send(VectorCanvas::AddShape(group, test_shape_type(), CanvasShape::Group)).await.unwrap();
        context.wait_for_idle(100).await;
        canvas.send(VectorCanvas::SetShapeParent(shape_2, CanvasShapeParent::Shape(group))).await.unwrap();
        context.wait_for_idle(100).await;

        // RemoveShape → ShapeChanged
        canvas.send(VectorCanvas::RemoveShape(shape_2)).await.unwrap();
        context.wait_for_idle(100).await;

        // RemoveBrush → no update
        canvas.send(VectorCanvas::RemoveBrush(brush_1)).await.unwrap();
        context.wait_for_idle(100).await;

        // RemoveLayer → LayerChanged
        canvas.send(VectorCanvas::RemoveLayer(layer_2)).await.unwrap();
        context.wait_for_idle(100).await;
    }, 1);

    // Work out the expected order for the two-shape case (HashSet iteration order is not guaranteed)
    let mut group_shapes = vec![shape_2, group];
    group_shapes.sort();

    TestBuilder::new()
        // AddLayer(layer_1) → LayerChanged
        .expect_message_matching(VectorCanvasUpdate::LayerChanged(vec![layer_1]), "AddLayer")
        // AddLayer(layer_2) → LayerChanged
        .expect_message_matching(VectorCanvasUpdate::LayerChanged(vec![layer_2]), "AddLayer (second)")
        // ReorderLayer(layer_2) → LayerChanged
        .expect_message_matching(VectorCanvasUpdate::LayerChanged(vec![layer_2]), "ReorderLayer")
        // AddShape(shape_1) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "AddShape")
        // SetShapeParent(shape_1, Layer(layer_1)) → LayerChanged + ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::LayerChanged(vec![layer_1]), "SetShapeParent to layer (layer update)")
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "SetShapeParent to layer (shape update)")
        // SetShapeDefinition(shape_1) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "SetShapeDefinition")
        // AddBrush → no update
        // AddShapeBrushes(shape_1) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "AddShapeBrushes")
        // SetProperty(Brush(brush_1)) → ShapeChanged (shape_1 has brush_1 attached)
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "SetProperty on brush")
        // RemoveProperty(Brush(brush_1)) → ShapeChanged (shape_1 has brush_1 attached)
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "RemoveProperty on brush")
        // RemoveShapeBrushes(shape_1) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "RemoveShapeBrushes")
        // SetProperty(Layer(layer_1)) → LayerChanged
        .expect_message_matching(VectorCanvasUpdate::LayerChanged(vec![layer_1]), "SetProperty on layer")
        // SetProperty(Shape(shape_1)) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "SetProperty on shape")
        // RemoveProperty(Shape(shape_1)) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_1]), "RemoveProperty on shape")
        // RemoveProperty(Layer(layer_1)) → LayerChanged
        .expect_message_matching(VectorCanvasUpdate::LayerChanged(vec![layer_1]), "RemoveProperty on layer")
        // AddShape(shape_2) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_2]), "AddShape (second)")
        // SetShapeParent(shape_2, Layer(layer_1)) → LayerChanged + ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::LayerChanged(vec![layer_1]), "SetShapeParent to layer (layer update, second)")
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_2]), "SetShapeParent to layer (shape update, second)")
        // ReorderShape(shape_2) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_2]), "ReorderShape")
        // AddShape(group) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![group]), "AddShape (group)")
        // SetShapeParent(shape_2, Shape(group)) → ShapeChanged (both shape_2 and group)
        .expect_message(move |update| match update { VectorCanvasUpdate::ShapeChanged(mut shapes) => { shapes.sort(); if shapes == group_shapes { Ok(()) } else { Err(format!("SetShapeParent to group: {:?} != {:?}", shapes, group_shapes)) } }, _ => Err("SetShapeParent to group, unexpected message type".into()) })
        // RemoveShape(shape_2) → ShapeChanged
        .expect_message_matching(VectorCanvasUpdate::ShapeChanged(vec![shape_2]), "RemoveShape")
        // RemoveBrush → no update
        // RemoveLayer(layer_2) → LayerChanged
        .expect_message_matching(VectorCanvasUpdate::LayerChanged(vec![layer_2]), "RemoveLayer")
        .run_in_scene(&scene, test_program);
}

#[test]
fn query_layer_frames_returns_shapes_for_correct_frame() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct TestResponse(Vec<VectorResponse>);

    impl SceneMessage for TestResponse { }

    let test_program    = SubProgramId::new();
    let query_program   = SubProgramId::new();

    let layer           = CanvasLayerId::new();
    let shape_1         = CanvasShapeId::new();
    let shape_2         = CanvasShapeId::new();

    let frame_1_time    = Duration::from_millis(0);
    let frame_2_time    = Duration::from_millis(1000);

    // Program that creates a layer with two frames, adds a shape to each, then queries each frame
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let _sqlite     = context.send::<SqliteCanvasRequest>(()).unwrap();
        let mut canvas  = context.send(()).unwrap();

        // Create a layer and two frames on it
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer, before_layer: None }).await.unwrap();
        canvas.send(VectorCanvas::AddFrame { frame_layer: layer, when: frame_1_time, length: Duration::from_millis(1000) }).await.unwrap();
        canvas.send(VectorCanvas::AddFrame { frame_layer: layer, when: frame_2_time, length: Duration::from_millis(1000) }).await.unwrap();

        // Add shapes and parent them to the layer at different frame times
        canvas.send(VectorCanvas::AddShape(shape_1, ShapeType::new("shape"), test_rect())).await.unwrap();
        canvas.send(VectorCanvas::SetShapeParent(shape_1, CanvasShapeParent::Layer(layer, frame_1_time))).await.ok();

        canvas.send(VectorCanvas::AddShape(shape_2, ShapeType::new("shape"), test_ellipse())).await.unwrap();
        canvas.send(VectorCanvas::SetShapeParent(shape_2, CanvasShapeParent::Layer(layer, frame_2_time))).await.ok();

        // Query at frame 1 time - should only return shape_1
        let frame_1_result = context.spawn_query(ReadCommand::default(), VectorQuery::Layers(().into(), vec![layer], frame_1_time), ()).unwrap();
        let frame_1_result = frame_1_result.collect::<Vec<_>>().await;

        context.send_message(TestResponse(frame_1_result)).await.unwrap();

        // Query at frame 2 time - should only return shape_2
        let frame_2_result = context.spawn_query(ReadCommand::default(), VectorQuery::Layers(().into(), vec![layer], frame_2_time), ()).unwrap();
        let frame_2_result = frame_2_result.collect::<Vec<_>>().await;

        context.send_message(TestResponse(frame_2_result)).await.unwrap();
    }, 1);

    let expected_frame_1 = vec![
        VectorResponse::Layer(layer, vec![]),
        VectorResponse::Shape(shape_1, test_rect(), ShapeType::new("shape"), vec![]),
    ];
    let expected_frame_2 = vec![
        VectorResponse::Layer(layer, vec![]),
        VectorResponse::Shape(shape_2, test_ellipse(), ShapeType::new("shape"), vec![]),
    ];

    // Run the test
    TestBuilder::new()
        .expect_message_matching(TestResponse(expected_frame_1), "Frame 1 should only contain shape_1")
        .expect_message_matching(TestResponse(expected_frame_2), "Frame 2 should only contain shape_2")
        .run_in_scene(&scene, test_program);
}

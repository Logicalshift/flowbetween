use super::*;
use super::super::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_scene::commands::*;

use futures::prelude::*;
use ::serde::*;

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

        // Set up some layers (layer2 vs layer1)
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
        VectorResponse::Shape(shape_1, vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(42))]),
        VectorResponse::Shape(shape_2, vec![(CanvasPropertyId::new("shape"), CanvasProperty::Int(43)), (CanvasPropertyId::new("brush1"), CanvasProperty::Int(45)), (CanvasPropertyId::new("brush2"), CanvasProperty::Int(49))]),
    ];

    // Run the test
    TestBuilder::new()
        .expect_message_matching(TestResponse(expected), "")
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
        canvas.send(VectorCanvas::SetShapeParent(shape_1, CanvasShapeParent::Layer(layer_1))).await.unwrap();
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
        canvas.send(VectorCanvas::SetShapeParent(shape_2, CanvasShapeParent::Layer(layer_1))).await.unwrap();
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

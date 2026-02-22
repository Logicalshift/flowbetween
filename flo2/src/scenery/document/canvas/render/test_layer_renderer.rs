use super::*;
use super::super::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas::*;
use ::serde::*;

#[test]
fn render_layer_empty() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct NumDrawingInstructions(usize);
    impl SceneMessage for NumDrawingInstructions {}

    let test_program  = SubProgramId::new();
    let query_program = SubProgramId::new();

    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let result = render_layer(vec![], FrameTime::ZERO, &context).await;

        context.send_message(NumDrawingInstructions(result.len())).await.unwrap();
    }, 1);

    TestBuilder::new()
        .expect_message_matching(NumDrawingInstructions(0), "render_layer with no entries should return an empty draw list")
        .run_in_scene(&scene, test_program);
}

#[test]
fn render_layer_flat_shapes() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct NumDrawingInstructions(usize);
    impl SceneMessage for NumDrawingInstructions {}

    let shape_type_1 = ShapeType::new("test::render_layer_flat::type_1");
    let shape_type_2 = ShapeType::new("test::render_layer_flat::type_2");

    // Renderer 1: generates 1 draw command per shape
    scene.add_subprogram(
        shape_type_1.render_program_id(),
        |input: InputStream<RenderShapesRequest>, context| async move {
            shape_renderer_program(input, context, |_shape, _time, drawing| {
                drawing.new_path();
            }).await;
        },
        0,
    );

    // Renderer 2: generates 3 draw commands per shape
    scene.add_subprogram(
        shape_type_2.render_program_id(),
        |input: InputStream<RenderShapesRequest>, context| async move {
            shape_renderer_program(input, context, |_shape, _time, drawing| {
                drawing.new_path();
                drawing.new_path();
                drawing.new_path();
            }).await;
        },
        0,
    );

    let test_program  = SubProgramId::new();
    let query_program = SubProgramId::new();

    // Call render_layer with a flat list of shapes (no groups) and check the total draw count
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let layer = vec![
            VectorResponse::Shape(CanvasShapeId::new(), CanvasShape::Group, FrameTime::ZERO, shape_type_1, vec![]),   // 1 draw
            VectorResponse::Shape(CanvasShapeId::new(), CanvasShape::Group, FrameTime::ZERO, shape_type_2, vec![]),   // 3 draws
            VectorResponse::Shape(CanvasShapeId::new(), CanvasShape::Group, FrameTime::ZERO, shape_type_1, vec![]),   // 1 draw
        ];

        let result = render_layer(layer, FrameTime::ZERO, &context).await;

        context.send_message(NumDrawingInstructions(result.len())).await.unwrap();
    }, 1);

    TestBuilder::new()
        .expect_message_matching(NumDrawingInstructions(5), "render_layer should concatenate the drawing instructions from all flat shapes in order")
        .run_in_scene(&scene, test_program);
}

#[test]
fn render_layer_nested_groups_passes_child_drawings_to_group_renderer() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct NumDrawingInstructions(usize);
    impl SceneMessage for NumDrawingInstructions {}

    let child_type = ShapeType::new("test::render_layer_groups::child");
    let group_type = ShapeType::new("test::render_layer_groups::group");

    // Child renderer: generates 2 draw commands per shape
    scene.add_subprogram(
        child_type.render_program_id(),
        |input: InputStream<RenderShapesRequest>, context| async move {
            shape_renderer_program(input, context, |_shape, _time, drawing| {
                drawing.new_path();
                drawing.new_path();
            }).await;
        },
        0,
    );

    // Group renderer: extends with the drawing instructions from all child shapes in shape.group.
    // (So the total count of shapes at the end is the number of )
    scene.add_subprogram(
        group_type.render_program_id(),
        |input: InputStream<RenderShapesRequest>, context| async move {
            shape_renderer_program(input, context, |shape, _time, drawing| {
                for (_child_shape, child_draws) in shape.group.iter() {
                    drawing.extend(child_draws.iter().cloned());
                }
            }).await;
        },
        0,
    );

    let test_program  = SubProgramId::new();
    let query_program = SubProgramId::new();

    // Build a layer with one group containing two child shapes
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let layer = vec![
            VectorResponse::Shape(CanvasShapeId::new(), CanvasShape::Group, FrameTime::ZERO, group_type, vec![]),
            VectorResponse::StartGroup,
            VectorResponse::Shape(CanvasShapeId::new(), CanvasShape::Group, FrameTime::ZERO, group_type, vec![]),
            VectorResponse::StartGroup,
            VectorResponse::Shape(CanvasShapeId::new(), CanvasShape::Group, FrameTime::ZERO, child_type, vec![]),
            VectorResponse::Shape(CanvasShapeId::new(), CanvasShape::Group, FrameTime::ZERO, child_type, vec![]),
            VectorResponse::EndGroup,
            VectorResponse::Shape(CanvasShapeId::new(), CanvasShape::Group, FrameTime::ZERO, child_type, vec![]),
            VectorResponse::EndGroup,
        ];

        let result = render_layer(layer, FrameTime::ZERO, &context).await;

        context.send_message(NumDrawingInstructions(result.len())).await.unwrap();
    }, 1);

    TestBuilder::new()
        .expect_message_matching(NumDrawingInstructions(6), "render_layer should populate shape.group with child drawings before calling the group renderer")
        .run_in_scene(&scene, test_program);
}

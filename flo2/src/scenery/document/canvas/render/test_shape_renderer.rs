use super::*;
use super::super::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas::*;
use ::serde::*;

use std::sync::*;

#[test]
fn render_shapes_dispatches_to_correct_renderer_and_preserves_order() {
    let scene = Scene::default();

    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct TestResponse(Vec<usize>);
    impl SceneMessage for TestResponse {}

    let shape_type_1 = ShapeType::new("test::render_shapes::type_1");
    let shape_type_2 = ShapeType::new("test::render_shapes::type_2");

    // Renderer 1: generates 1 draw command per shape
    scene.add_subprogram(
        shape_type_1.render_program_id(),
        |input: InputStream<RenderShapesRequest>, context| async move {
            shape_renderer_program(input, context, |_shape, drawing| {
                drawing.new_path();
            }).await;
        },
        0,
    );

    // Renderer 2: generates 3 draw commands per shape
    scene.add_subprogram(
        shape_type_2.render_program_id(),
        |input: InputStream<RenderShapesRequest>, context| async move {
            shape_renderer_program(input, context, |_shape, drawing| {
                drawing.new_path();
                drawing.new_path();
                drawing.new_path();
            }).await;
        },
        0,
    );

    let test_program  = SubProgramId::new();
    let query_program = SubProgramId::new();

    // Program that creates shapes with interleaved types, calls render_shapes(), and sends the result
    scene.add_subprogram(query_program, move |_input: InputStream<()>, context| async move {
        let make_shape = |shape_type: ShapeType| Arc::new(ShapeWithProperties {
            shape:      CanvasShape::Group,
            shape_type: shape_type,
            properties: Arc::new(vec![]),
            group:      vec![],
        });

        // Interleave shapes of different types: the result order must match the input order
        let shapes = vec![
            make_shape(shape_type_1),   // should produce 1 draw command (renderer 1)
            make_shape(shape_type_2),   // should produce 3 draw commands (renderer 2)
            make_shape(shape_type_1),   // should produce 1 draw command (renderer 1)
            make_shape(shape_type_2),   // should produce 3 draw commands (renderer 2)
        ];

        let result      = render_shapes(shapes.into_iter(), &context).await;
        let draw_counts = result.iter().map(|d| d.len()).collect::<Vec<_>>();

        context.send_message(TestResponse(draw_counts)).await.unwrap();
    }, 1);

    TestBuilder::new()
        .expect_message_matching(
            TestResponse(vec![1, 3, 1, 3]),
            "render_shapes should dispatch to the correct renderer for each shape type and return results in input order",
        )
        .run_in_scene(&scene, test_program);
}

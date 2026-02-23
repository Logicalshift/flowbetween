use super::*;
use super::super::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas::scenery::*;
use futures::prelude::*;

#[test]
fn generates_drawing_on_startup() {
    let scene = Scene::default();

    let test_program  = SubProgramId::new();
    let setup_program = SubProgramId::new();

    let layer_id = CanvasLayerId::new();
    let shape_id = CanvasShapeId::new();

    // Set up the canvas with a layer and a shape, then start the canvas renderer
    scene.add_subprogram(setup_program, move |_input: InputStream<()>, context| async move {
        // Initialise the sqlite canvas program (stores document data and handles VectorQuery routing)
        let _sqlite = context.send::<SqliteCanvasRequest>(()).unwrap();
        let mut canvas = context.send(()).unwrap();

        // Add a layer and a shape to give the renderer something to draw
        canvas.send(VectorCanvas::AddLayer { new_layer_id: layer_id, before_layer: None }).await.unwrap();
        canvas.send(VectorCanvas::AddShape(shape_id, ShapeType::default(), CanvasShape::Rectangle(CanvasRectangle {
            min: CanvasPoint { x: 0.0, y: 0.0 },
            max: CanvasPoint { x: 100.0, y: 100.0 },
        }))).await.unwrap();
        canvas.send(VectorCanvas::SetShapeParent(shape_id, CanvasShapeParent::Layer(layer_id, FrameTime::ZERO))).await.unwrap();

        // Initialise the canvas render program: it starts with need_redraw=true and requests an
        // idle event, so it will produce a DrawingRequest as soon as the scene becomes idle
        let _canvas_render = context.send::<CanvasRender>(()).unwrap();
    }, 1);

    // The canvas renderer should produce a DrawingRequest once the scene is idle
    TestBuilder::new()
        .expect_message(|msg: DrawingRequest| {
            match msg {
                DrawingRequest::Draw(instructions) => {
                    if instructions.is_empty() {
                        Err("Canvas renderer produced an empty DrawingRequest on startup".into())
                    } else {
                        Ok(())
                    }
                }
            }
        })
        .run_in_scene(&scene, test_program);
}

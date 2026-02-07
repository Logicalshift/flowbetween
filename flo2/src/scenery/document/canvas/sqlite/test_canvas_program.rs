use super::*;

use super::super::layer::*;
use super::super::queries::*;
use super::super::vector_editor::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_scene::commands::*;

use futures::prelude::*;
use ::serde::*;

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

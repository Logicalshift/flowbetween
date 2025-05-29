mod scenery;

use crate::scenery::app::*;
use crate::scenery::document::*;

use flo_draw::*;
use flo_scene::*;

use std::sync::*;

fn main() {
    with_2d_graphics(|| {
        use futures::prelude::*;
        use futures::executor;
        use futures::channel::oneshot;
        use flo_draw::draw_scene::*;

        // Create the application scene (this also starts it running)
        let app_scene = flo_draw_scene_context();

        // Run the main application program
        let (stop_send, stop_recv) = oneshot::channel();

        // Run the flowbetween app
        let scene = Arc::clone(&app_scene);
        app_scene.add_subprogram(FlowBetween::default_target().target_sub_program().unwrap(), move |input, context| flowbetween(scene, input, context), 20);

        // Run a subprogram we use to keep things alive and shutdown when we're done
        app_scene.add_subprogram(SubProgramId::called("flowbetween::main"), |events: InputStream<()>, context| async move { 
            let mut events = events;

            context.send_message(FlowBetween::CreateEmptyDocument(DocumentId::new())).await.unwrap();

            while let Some(_evt) = events.next().await {

            }

            stop_send.send(()).unwrap();
        }, 1);

        // Wait for the main subprogram to stop (whole application will stop once this function ends, and all the windows are closed)
        executor::block_on(async move { stop_recv.await.unwrap(); });
    });
}

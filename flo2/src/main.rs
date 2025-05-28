mod scenery;

use flo_draw::*;
use flo_scene::*;

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

        app_scene.add_subprogram(SubProgramId::called("flowbetween::main"), |events: InputStream<()>, _context| async move { 
            let mut events = events;

            while let Some(evt) = events.next().await {

            }

            stop_send.send(());
        }, 1);

        // Wait for the main subprogram to stop (whole application will stop once this function ends, and all the windows are closed)
        executor::block_on(async move { stop_recv.await; });
    });
}

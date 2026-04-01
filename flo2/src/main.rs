// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod scenery;

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

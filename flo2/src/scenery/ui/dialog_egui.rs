use super::dialog::*;
use super::focus::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas as canvas;
use flo_draw::canvas::scenery::*;

use futures::prelude::*;

use std::sync::*;

///
/// Defines dialog behavior using 
///
pub async fn dialog_egui(input: InputStream<Dialog>, context: SceneContext) {
    use canvas::{Draw, LayerId};

    // Create a namespace for the dialog graphics
    let dialog_subprogram   = context.current_program_id().unwrap();
    let dialog_namespace    = canvas::NamespaceId::new();

    // We'll be sending drawing requests
    let mut drawing         = context.send::<DrawingRequest>(()).unwrap();
    let mut idle_requests   = context.send::<IdleRequest>(()).unwrap();

    // Set up the dialog layer (it'll go on top of anything else in the drawing at the moment)
    drawing.send(DrawingRequest::Draw(Arc::new(vec![
        Draw::PushState,
        Draw::Namespace(dialog_namespace),
        Draw::Layer(LayerId(0)),
        Draw::PopState,
    ]))).await.ok();

    // Set to true if we've requested an idle event
    let mut awaiting_idle   = false;
    let mut input           = input;

    while let Some(input) = input.next().await {
        use Dialog::*;

        match input {
            Idle => {
                awaiting_idle = false;

                // TODO: run the egui context
            },

            FocusEvent(event) => {
                // TODO: process the event

                // Request an idle event (we'll use this to run the egui)
                if !awaiting_idle {
                    awaiting_idle = true;
                    idle_requests.send(IdleRequest::WhenIdle(dialog_subprogram)).await.ok();
                }
            }
        }
    }
}

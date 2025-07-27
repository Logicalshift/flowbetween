use super::binding_tracker::*;

use flo_binding::*;
use flo_binding::binding_context::*;
use flo_draw::canvas::scenery::*;
use flo_draw::canvas::*;
use flo_scene::*;
use flo_scene::programs::*;

use ::serde::*;
use futures::prelude::*;

use std::sync::*;

///
/// The render binding subprogram takes a binding to some render instructions, a namespace and a layer, and
/// generates drawing requests for that layer whenever that binding is updated. Optionally it can take the 
/// identifier of a parent program, and stop and clear the layer if that program ever quits.
///
/// Rendering is done in scene idle cycles: this ensures that if many updates arrive only a single drawing
/// event will be generated.
///
/// This subprogram is very useful for drawing dynamically as it removes the need for a program to generate
/// its own rendering events.
///
pub fn render_binding_program(input: InputStream<RenderBinding>, context: SceneContext, render_layer: (NamespaceId, LayerId), parent_program: Option<SubProgramId>, rendering: BindRef<Vec<Draw>>) -> impl Send + Future<Output=()> {
    async move {
        let mut input = input;

        // TODO: would be nice to be able to do this with sprites as well as layers (need a custom type for RenderLayer I think)

        // Need to know the program ID we've been assigned
        let our_program_id = context.current_program_id().unwrap();

        // Connect to the idle request program
        let mut idle_requests = context.send::<IdleRequest>(()).unwrap();

        // When the canvas is rendered, we send a drawing request
        let mut draw_requests = context.send::<DrawingRequest>(()).unwrap();

        // We want scene updates so we can stop if the parent program stops
        if parent_program.is_some() {
            context.send_message(SceneControl::Subscribe(our_program_id.into())).await.ok();
        }

        // This releasable is what monitors the render binding
        let mut render_binding_tracker;

        // Render the initial scene, and set up the tracker
        render_binding_tracker  = Some(rendering.when_changed(NotifySubprogram::send(RenderBinding::DrawingChanged, &context, our_program_id)));
        let initial_drawing     = rendering.get();

        let mut initial_drawing = initial_drawing;
        initial_drawing.splice(0..0, vec![
            Draw::PushState,
            Draw::Namespace(render_layer.0),
            Draw::Layer(render_layer.1),
            Draw::ClearLayer,
        ]);
        initial_drawing.push(Draw::PopState);

        draw_requests.send(DrawingRequest::Draw(Arc::new(initial_drawing))).await.ok();

        // Monitor for events and render the drawing when needed
        let mut requested_idle = false;

        while let Some(event) = input.next().await {
            use RenderBinding::*;

            match event {
                DrawingChanged => {
                    // Request an idle notification if one isn't already waiting
                    if !requested_idle {
                        // Clear the tracker
                        if let Some(mut render_binding_tracker) = render_binding_tracker.take() {
                            render_binding_tracker.done();
                        }

                        // Request an idle event
                        idle_requests.send(IdleRequest::WhenIdle(our_program_id)).await.ok();
                        requested_idle = true;
                    }
                }

                Idle => {
                    // The idle event is triggered after the drawing is changed: we need to re-render the drawing
                    requested_idle = false;

                    // Render the drawing, and wait for the rendering to change again
                    render_binding_tracker  = Some(rendering.when_changed(NotifySubprogram::send(RenderBinding::DrawingChanged, &context, our_program_id)));
                    let drawing             = rendering.get();

                    let mut drawing = drawing;
                    drawing.splice(0..0, vec![
                        Draw::PushState,
                        Draw::Namespace(render_layer.0),
                        Draw::Layer(render_layer.1),
                        Draw::ClearLayer,
                    ]);
                    drawing.push(Draw::PopState);

                    draw_requests.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
                }

                Update(SceneUpdate::Stopped(program_id)) => {
                    if Some(program_id) == parent_program {
                        // Stop anything from notifying us
                        if let Some(mut render_binding_tracker) = render_binding_tracker.take() {
                            render_binding_tracker.done();
                        }

                        // When the parent program stops, clear the layer that we were rendering
                        draw_requests.send(DrawingRequest::Draw(Arc::new(vec![
                            Draw::PushState,
                            Draw::Namespace(render_layer.0),
                            Draw::Layer(render_layer.1),
                            Draw::ClearLayer,
                            Draw::PopState,
                        ]))).await.ok();

                        // Stop running if the parent program has been stopped
                        break;
                    }
                }

                Update(_) => { /* Other updates are ignored */}
            }
        }
    }
}

///
/// Messages that can be sent to the render binding program
///
/// There's no need to send these manually, they're all to do with managing the rendering of the layer,
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RenderBinding {
    /// The scene is idle so any changes can be rendered
    Idle,

    /// The drawing has changed, so we should request an idle event to perform a render
    DrawingChanged,

    /// Something about the scene has changed
    Update(SceneUpdate),
}

impl SceneMessage for RenderBinding {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        // The render binding programs can receive idle notifications and scene updates
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|scene_updates| scene_updates.map(|update| RenderBinding::Update(update)))), (), StreamId::with_message_type::<SceneUpdate>()).ok();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|idle_updates| idle_updates.map(|_: IdleNotification| RenderBinding::Idle))), (), StreamId::with_message_type::<IdleNotification>()).ok();
    }
}

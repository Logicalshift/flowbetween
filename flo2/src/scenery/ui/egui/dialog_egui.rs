use super::events::*;
use super::draw::*;
use super::state::*;

use crate::scenery::ui::dialog::*;
use crate::scenery::ui::focus::*;
use crate::scenery::ui::ui_path::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas as canvas;
use flo_draw::canvas::scenery::*;
use flo_curves::bezier::path::*;
use flo_binding::*;
use flo_binding::binding_context::*;

use egui;
use egui::epaint;
use serde::*;
use futures::prelude::*;
use futures::select;
use futures::task;

use std::sync::*;
use std::time::{Instant};

///
/// Request sent to an egui dialog
///
#[derive(Serialize, Deserialize, Clone)]
pub (crate) enum EguiDialogRequest {
    /// Event indicating that the scene is idle
    Idle,

    /// Event indicating that the bindings have changed
    BindingsChanged,

    /// Event from the focus subprogram (used to direct events to the dialog program)
    FocusEvent(FocusEvent),

    /// Other dialog request (the egui dialog programs manage one dialog each, so they )
    Dialog(Dialog),
}

///
/// State used to wake the egui dialog when there's a change to the bindings
///
struct ChangeWakeState {
    awake: bool,
    waker: Option<task::Waker>,
}

///
/// Creates a future that wakes up when the ChangeWakeState is awoken
///
#[inline]
fn change_waker_future(state: &Arc<Mutex<ChangeWakeState>>) -> impl Future<Output=EguiDialogRequest> {
    let state = Arc::clone(state);

    future::poll_fn(move |context| {
        let mut state = state.lock().unwrap();

        if state.awake {
            task::Poll::Ready(EguiDialogRequest::BindingsChanged)
        } else {
            state.waker = Some(context.waker().clone());
            task::Poll::Pending
        }
    })
}

///
/// Defines dialog behavior by using egui (with rendering via flo_canvas requests)
///
pub (crate) async fn dialog_egui(input: InputStream<EguiDialogRequest>, context: SceneContext, dialog_namespace: canvas::NamespaceId, dialog_layer: canvas::LayerId, bounds: (UiPoint, UiPoint)) {
    use canvas::{Draw, LayerId};

    // Create a namespace for the dialog graphics
    let dialog_subprogram   = context.current_program_id().unwrap();
    let dialog_namespace    = canvas::NamespaceId::new();

    let region = BezierPathBuilder::<UiPath>::start(bounds.0)
        .line_to(UiPoint(bounds.0.0, bounds.1.1))
        .line_to(bounds.1)
        .line_to(UiPoint(bounds.1.0, bounds.0.1))
        .line_to(bounds.0)
        .build();
    context.send_message(Focus::ClaimRegion { program: dialog_subprogram, region: vec![region], z_index: 0 }).await.ok();

    let mut dialog_state = EguiDialogState::new();

    // We'll be sending drawing requests
    let mut drawing         = context.send::<DrawingRequest>(()).unwrap();
    let mut idle_requests   = context.send::<IdleRequest>(()).unwrap();

    // The wake state is used to 
    let mut change_wake_state = Arc::new(Mutex::new(ChangeWakeState { awake: false, waker: None }));

    // Set up the dialog layer (it'll go on top of anything else in the drawing at the moment)
    drawing.send(DrawingRequest::Draw(Arc::new(vec![
        Draw::PushState,
        Draw::Namespace(dialog_namespace),
        Draw::Layer(LayerId(0)),
        Draw::PopState,
    ]))).await.ok();

    // Releasable used to stop monitoring out-of-date binding
    let mut binding_monitor = None;

    // Set up the egui context
    let egui_context        = egui::Context::default();
    let mut pending_input   = egui::RawInput::default();
    let start_time          = Instant::now();

    // TODO: size is where this dialog appears on screen (if we use one viewport per dialog)
    pending_input.screen_rect = Some(egui::Rect { min: egui::Pos2 { x: bounds.0.0 as _, y: bounds.0.1 as _ }, max: egui::Pos2 { x: bounds.1.0 as _, y: bounds.1.1 as _ } });

    // Request an idle event after startup so we render any UI we need to
    idle_requests.send(IdleRequest::WhenIdle(dialog_subprogram)).await.ok();

    // Set to true if we've requested an idle event
    let mut awaiting_idle   = true;
    let mut input           = input;

    while let Some(input) = select! { evt = input.next().fuse() => evt, evt = change_waker_future(&change_wake_state).fuse() => Some(evt) } {
        use EguiDialogRequest::*;

        match input {
            Idle => {
                use std::mem;

                // Set the time on the events
                let since_start = start_time.elapsed();
                let since_start = since_start.as_micros();
                let since_start = (since_start as f64)/1_000_000.0;

                pending_input.time = Some(since_start);

                // Not waiting for any more idle events
                awaiting_idle = false;

                // Cycle the pending input
                let mut new_input   = egui::RawInput::default();
                new_input.modifiers = pending_input.modifiers;
                mem::swap(&mut new_input, &mut pending_input);

                // Run the egui context
                let mut events  = None;
                binding_monitor = None;

                let output = egui_context.run(new_input, |ctxt| {
                    // Use a binding context to monitor the bindings
                    let (_, deps) = BindingContext::bind(|| {
                        // Run the UI request
                        events = Some(dialog_state.run(ctxt, bounds));
                    });

                    // When any of the bindings change, create a notification to wake us up
                    let change_wake_state = change_wake_state.clone();
                    binding_monitor = Some(deps.when_changed(notify(move || {
                        // Mark as changed and take the waker from the state
                        let waker = {
                            let mut change_wake_state = change_wake_state.lock().unwrap();

                            change_wake_state.awake = true;
                            change_wake_state.waker.take()
                        };

                        // If we got a waker, wake it up (waking up the dialog to be redrawn)
                        if let Some(waker) = waker {
                            waker.wake();
                        }
                    })));
                });

                // Process the output, generating draw events
                process_texture_output(&output, &mut drawing, dialog_namespace).await;
                process_drawing_output(&output, &mut drawing, dialog_namespace).await;
            },

            BindingsChanged => {
                // Reset the change state to be asleep again so we don't immediately wake up again
                change_wake_state.lock().unwrap().awake = false;

                // Requesting an idle event will cause the dialog to be redrawn with any changes
                if !awaiting_idle {
                    awaiting_idle = true;
                    idle_requests.send(IdleRequest::WhenIdle(dialog_subprogram)).await.ok();
                }
            }

            FocusEvent(focus_event) => {
                // Process the event
                use crate::scenery::ui::focus::{FocusEvent};

                match focus_event {
                    FocusEvent::Event(_control, event)  => { convert_events(&mut pending_input, event); }
                    FocusEvent::Focused(_control)       => { }
                    FocusEvent::Unfocused(_control)     => { }
                }

                // Request an idle event (we'll use this to run the egui)
                if !awaiting_idle {
                    awaiting_idle = true;
                    idle_requests.send(IdleRequest::WhenIdle(dialog_subprogram)).await.ok();
                }
            }

            Dialog(other_dialog_event) => {
                // Update the state of the dialog according to this event
                dialog_state.update_state(&other_dialog_event);

                // Request an idle event (we'll use this to run the egui)
                if !awaiting_idle {
                    awaiting_idle = true;
                    idle_requests.send(IdleRequest::WhenIdle(dialog_subprogram)).await.ok();
                }
            }
        }
    }
}

impl SceneMessage for EguiDialogRequest {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        // Set up filters for the events that an EguiDialog can handle
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|focus_events| focus_events.map(|focus| EguiDialogRequest::FocusEvent(focus)))), (), StreamId::with_message_type::<FocusEvent>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|idle_events| idle_events.map(|_idle: IdleNotification| EguiDialogRequest::Idle))), (), StreamId::with_message_type::<IdleNotification>()).unwrap();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|dialog_events| dialog_events.map(|dialog| EguiDialogRequest::Dialog(dialog)))), (), StreamId::with_message_type::<Dialog>()).unwrap();
    }
}

///
/// Processes the drawing instructions in the output from egui into flo_draw canvas instructions
///
async fn process_drawing_output(output: &egui::FullOutput, drawing_target: &mut OutputSink<DrawingRequest>, namespace: canvas::NamespaceId) {
    use canvas::{Draw, LayerId};

    let mut drawing = vec![];

    // Start by selecting the namespace and storing the state
    drawing.extend([
        Draw::PushState,
        Draw::Namespace(namespace),
        Draw::Layer(LayerId(0)),
        Draw::ClearLayer,
    ]);
    let initial_len = drawing.len();

    // Process drawing commands
    for shape in output.shapes.iter() {
        draw_shape(&shape.shape, &mut drawing);
    }

    // Free any textures that aren't used any more
    output.textures_delta.free.iter()
        .for_each(|texture_id| {
            drawing.push(Draw::Texture(canvas_texture_id(texture_id), canvas::TextureOp::Free));
        });

    // Send the drawing if we generated any drawing instructions
    if drawing.len() > initial_len {
        drawing.extend([
            Draw::PopState,
        ]);

        drawing_target.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
    }
}

///
/// Returns the size and bytes ready to send to the canvas for a texture
///
fn canvas_texture_bytes(image: &epaint::ImageData) -> (usize, usize, Arc<Vec<u8>>) {
    let (width, height, bytes) = match image {
        epaint::ImageData::Color(color_image) => {
            let bytes = color_image.pixels.iter()
                .flat_map(|pixel| {
                    let (r, g, b, a) = pixel.to_tuple();
                    [r, g, b, a]
                })
                .collect();

            (color_image.size[0], color_image.size[1], bytes)
        }

        epaint::ImageData::Font(font_image) => {
            let bytes = font_image.srgba_pixels(None)
                .flat_map(|pixel| {
                    let (r, g, b, a) = pixel.to_tuple();
                    [r, g, b, a]
                })
                .collect();

            (font_image.size[0], font_image.size[1], bytes)
        }
    };

    (width, height, Arc::new(bytes))
}

///
/// Processes the texture instructions in the output from egui into flo_draw canvas instructions
///
async fn process_texture_output(output: &egui::FullOutput, drawing_target: &mut OutputSink<DrawingRequest>, namespace: canvas::NamespaceId) {
    use canvas::{Draw, LayerId, TextureOp, TexturePosition, TextureSize, TextureFormat};

    let mut drawing = vec![];

    // Start by selecting the namespace and storing the state
    drawing.extend([
        Draw::PushState,
        Draw::Namespace(namespace),
        Draw::Layer(LayerId(0)),
        Draw::ClearLayer,
    ]);
    let initial_len = drawing.len();

    // Deal with any textures to set during the following drawing instructions (free instructions have to be dealt with after drawing)
    output.textures_delta.set.iter()
        .for_each(|(texture_id, image_delta)| { 
            // Convert the texture ID
            let texture_id              = canvas_texture_id(texture_id);
            let (width, height, bytes)  = canvas_texture_bytes(&image_delta.image);

            if let Some(pos) = image_delta.pos {
                // Update an existing texture
                drawing.extend([
                    Draw::Texture(texture_id, TextureOp::SetBytes(TexturePosition(pos[0] as _, pos[1] as _), TextureSize(width as _, height as _), bytes)),
                ]);
            } else {
                // Create a new texture
                // TODO: hard coding the texture size to be 2048, 2048 as that seems to be what's expected for the font
                drawing.extend([
                    Draw::Texture(texture_id, TextureOp::Create(TextureSize(width as _, height as _), TextureFormat::Rgba)),
                    Draw::Texture(texture_id, TextureOp::SetBytes(TexturePosition(0, 0), TextureSize(width as _, height as _), bytes)),
                ]);
            }
        });

    // Send the drawing if we generated any drawing instructions
    if drawing.len() > initial_len {
        drawing.extend([
            Draw::PopState,
        ]);

        drawing_target.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
    }
}

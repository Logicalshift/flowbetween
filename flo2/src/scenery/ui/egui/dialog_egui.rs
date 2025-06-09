use super::key::*;
use super::events::*;
use super::draw::*;

use crate::scenery::ui::dialog::*;
use crate::scenery::ui::focus::*;
use crate::scenery::ui::ui_path::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw as draw;
use flo_draw::canvas as canvas;
use flo_draw::canvas::scenery::*;
use flo_draw::canvas::{GraphicsContext, GraphicsPrimitives};
use flo_curves::geo::*;
use flo_curves::bezier::path::*;

use egui;
use egui::epaint;
use futures::prelude::*;

use std::sync::*;
use std::time::{Duration, Instant};

///
/// Defines dialog behavior by using egui (with rendering via flo_canvas requests)
///
pub async fn dialog_egui(input: InputStream<Dialog>, context: SceneContext) {
    use canvas::{Draw, LayerId};

    // Create a namespace for the dialog graphics
    let dialog_subprogram   = context.current_program_id().unwrap();
    let dialog_namespace    = canvas::NamespaceId::new();

    // TODO: this claims a large region so we get events (but we'll only want events for things actually covered by a dialog when we're done)
    let region = BezierPathBuilder::<UiPath>::start(UiPoint(0.0, 0.0))
        .line_to(UiPoint(0.0, 1000.0))
        .line_to(UiPoint(1000.0, 1000.0))
        .line_to(UiPoint(1000.0, 0.0))
        .line_to(UiPoint(0.0, 0.0))
        .build();
    context.send_message(Focus::ClaimRegion { program: dialog_subprogram, region: vec![region], z_index: 0 }).await.ok();

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

    // Set up the EGUI context
    let egui_context        = egui::Context::default();
    let mut pending_input   = egui::RawInput::default();
    let start_time          = Instant::now();

    // TODO: size is where this dialog appears on screen (if we use one viewport per dialog)
    pending_input.screen_rect = Some(egui::Rect { min: egui::Pos2 { x: 0.0, y: 0.0 }, max: egui::Pos2 { x: 1000.0, y: 1000.0 } });

    // Request an idle event after startup so we render any UI we need to
    idle_requests.send(IdleRequest::WhenIdle(dialog_subprogram)).await.ok();

    // Set to true if we've requested an idle event
    let mut awaiting_idle   = true;
    let mut input           = input;

    while let Some(input) = input.next().await {
        use Dialog::*;

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
                let output = egui_context.run(new_input, |ctxt| {
                    // TODO: simple test of the dialog
                    egui::CentralPanel::default().show(&ctxt, |ui| {
                        ui.add(egui::Label::new("Hello World!"));
                        ui.label("A shorter and more convenient way to add a label.");
                        if ui.button("Click me").clicked() {
                            // take some action here
                        }
                        let mut checked = true;
                        ui.checkbox(&mut checked, "Test");
                    });
                });

                // Process the output, generating draw events
                process_texture_output(&output, &mut drawing, dialog_namespace).await;
                process_drawing_output(&output, &mut drawing, dialog_namespace).await;
            },

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
        }
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

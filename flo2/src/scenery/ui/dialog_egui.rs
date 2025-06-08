use super::dialog::*;
use super::focus::*;
use super::ui_path::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw as draw;
use flo_draw::canvas as canvas;
use flo_draw::canvas::scenery::*;
use flo_draw::canvas::{GraphicsContext, GraphicsPrimitives};
use flo_curves::geo::*;

use egui;
use egui::epaint;
use futures::prelude::*;

use std::sync::*;

///
/// Defines dialog behavior by using egui (with rendering via flo_canvas requests)
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

    // Set up the EGUI context
    let egui_context        = egui::Context::default();
    let mut pending_input   = egui::RawInput::default();

    // Set to true if we've requested an idle event
    let mut awaiting_idle   = false;
    let mut input           = input;

    while let Some(input) = input.next().await {
        use Dialog::*;

        match input {
            Idle => {
                use std::mem;

                // Not waiting for any more idle events
                awaiting_idle = false;

                // Cycle the pending input
                let mut new_input   = egui::RawInput::default();
                new_input.modifiers = pending_input.modifiers;
                mem::swap(&mut new_input, &mut pending_input);

                // Run the egui context
                let output = egui_context.run(new_input, |_ctxt| { });

                // Process the output, generating draw events
                process_drawing_output(&output, &mut drawing, dialog_namespace).await;
            },

            FocusEvent(focus_event) => {
                // Process the event
                use super::focus::{FocusEvent};

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
/// Converts a keypress from flo_draw to an egui Key object, if there's an equivalent
///
fn convert_key(draw_key: draw::Key) -> Option<egui::Key> {
    use draw::Key::*;

    match draw_key {
        Unknown             => None,

        ModifierShift       => None,
        ModifierCtrl        => None,
        ModifierAlt         => None,
        ModifierMeta        => None,
        ModifierSuper       => None,
        ModifierHyper       => None,

        KeyTab              => Some(egui::Key::Tab),

        KeyA                => Some(egui::Key::A),
        KeyB                => Some(egui::Key::B),
        KeyC                => Some(egui::Key::C),
        KeyD                => Some(egui::Key::D),
        KeyE                => Some(egui::Key::E),
        KeyF                => Some(egui::Key::F),
        KeyG                => Some(egui::Key::G),
        KeyH                => Some(egui::Key::H),
        KeyI                => Some(egui::Key::I),
        KeyJ                => Some(egui::Key::J),
        KeyK                => Some(egui::Key::K),
        KeyL                => Some(egui::Key::L),
        KeyM                => Some(egui::Key::M),
        KeyN                => Some(egui::Key::N),
        KeyO                => Some(egui::Key::O),
        KeyP                => Some(egui::Key::P),
        KeyQ                => Some(egui::Key::Q),
        KeyR                => Some(egui::Key::R),
        KeyS                => Some(egui::Key::S),
        KeyT                => Some(egui::Key::T),
        KeyU                => Some(egui::Key::U),
        KeyV                => Some(egui::Key::V),
        KeyW                => Some(egui::Key::W),
        KeyX                => Some(egui::Key::X),
        KeyY                => Some(egui::Key::Y),
        KeyZ                => Some(egui::Key::Z),
        
        Key1                => Some(egui::Key::Num1),
        Key2                => Some(egui::Key::Num2),
        Key3                => Some(egui::Key::Num3),
        Key4                => Some(egui::Key::Num4),
        Key5                => Some(egui::Key::Num5),
        Key6                => Some(egui::Key::Num6),
        Key7                => Some(egui::Key::Num7),
        Key8                => Some(egui::Key::Num8),
        Key9                => Some(egui::Key::Num9),
        Key0                => Some(egui::Key::Num0),

        KeyUp               => Some(egui::Key::ArrowUp),
        KeyDown             => Some(egui::Key::ArrowDown),
        KeyLeft             => Some(egui::Key::ArrowLeft),
        KeyRight            => Some(egui::Key::ArrowRight),

        KeyBackslash        => Some(egui::Key::Backslash),
        KeyForwardslash     => Some(egui::Key::Slash),
        KeyBacktick         => Some(egui::Key::Backtick),
        KeyComma            => Some(egui::Key::Comma),
        KeyFullstop         => Some(egui::Key::Period),
        KeySemicolon        => Some(egui::Key::Semicolon),
        KeyQuote            => Some(egui::Key::Quote),
        KeyMinus            => Some(egui::Key::Minus),
        KeyEquals           => Some(egui::Key::Equals),

        KeySpace            => Some(egui::Key::Space),
        KeyEscape           => Some(egui::Key::Escape),
        KeyInsert           => Some(egui::Key::Insert),
        KeyHome             => Some(egui::Key::Home),
        KeyPgUp             => Some(egui::Key::PageUp),
        KeyDelete           => Some(egui::Key::Delete),
        KeyEnd              => Some(egui::Key::End),
        KeyPgDown           => Some(egui::Key::PageDown),
        KeyBackspace        => Some(egui::Key::Backspace),
        KeyEnter            => Some(egui::Key::Enter),

        KeyF1               => Some(egui::Key::F1),
        KeyF2               => Some(egui::Key::F2),
        KeyF3               => Some(egui::Key::F3),
        KeyF4               => Some(egui::Key::F4),
        KeyF5               => Some(egui::Key::F5),
        KeyF6               => Some(egui::Key::F6),
        KeyF7               => Some(egui::Key::F7),
        KeyF8               => Some(egui::Key::F8),
        KeyF9               => Some(egui::Key::F9),
        KeyF10              => Some(egui::Key::F10),
        KeyF11              => Some(egui::Key::F11),
        KeyF12              => Some(egui::Key::F12),
        KeyF13              => Some(egui::Key::F13),
        KeyF14              => Some(egui::Key::F14),
        KeyF15              => Some(egui::Key::F15),
        KeyF16              => Some(egui::Key::F16),

        KeyNumpad0          => Some(egui::Key::Num0),
        KeyNumpad1          => Some(egui::Key::Num1),
        KeyNumpad2          => Some(egui::Key::Num2),
        KeyNumpad3          => Some(egui::Key::Num3),
        KeyNumpad4          => Some(egui::Key::Num4),
        KeyNumpad5          => Some(egui::Key::Num5),
        KeyNumpad6          => Some(egui::Key::Num6),
        KeyNumpad7          => Some(egui::Key::Num7),
        KeyNumpad8          => Some(egui::Key::Num8),
        KeyNumpad9          => Some(egui::Key::Num9),
        KeyNumpadDivide     => Some(egui::Key::Slash),
        KeyNumpadMultiply   => None,
        KeyNumpadMinus      => Some(egui::Key::Minus),
        KeyNumpadAdd        => Some(egui::Key::Plus),
        KeyNumpadEnter      => Some(egui::Key::Enter),
        KeyNumpadDecimal    => Some(egui::Key::Period),
    }
}

///
/// Converts a button press from flo_draw to egui
///
fn convert_button(button: draw::Button) -> egui::PointerButton {
    use draw::Button;

    match button {
        Button::Left        => egui::PointerButton::Primary,
        Button::Right       => egui::PointerButton::Secondary,
        Button::Middle      => egui::PointerButton::Middle,
        Button::Other(0)    => egui::PointerButton::Extra1,
        Button::Other(1)    => egui::PointerButton::Extra2,
        Button::Other(_)    => egui::PointerButton::Extra2,   
    }
}

///
/// Converts DrawEvents to egui events
///
fn convert_events(pending_input: &mut egui::RawInput, event: draw::DrawEvent) {
    use draw::DrawEvent::*;
    use draw::PointerAction;

    match event {
        Redraw              => { }
        NewFrame            => { }
        Scale(new_scale)    => { }  // TODO: think this affects egui's texture rendering
        Resize(_, _)        => { }
        CanvasTransform(_)  => { }
        Closed              => { }

        // Pointer actions
        Pointer(PointerAction::Move, _, pointer_state) => {
            if let Some(location) = pointer_state.location_in_canvas {
                pending_input.events.push(egui::Event::PointerMoved(egui::Pos2 { x: location.0 as _, y: location.1 as _ }));
            }
        }

        Pointer(PointerAction::ButtonDown, _, pointer_state) => {
            if let Some(location) = pointer_state.location_in_canvas {
                for button in pointer_state.buttons.iter() {
                    pending_input.events.push(egui::Event::PointerButton {
                        pos:        egui::Pos2 { x: location.0 as _, y: location.1 as _ },
                        button:     convert_button(*button),
                        pressed:    true,
                        modifiers:  pending_input.modifiers,
                    });
                }
            }
        }

        Pointer(PointerAction::ButtonUp, _, pointer_state) => {
            if let Some(location) = pointer_state.location_in_canvas {
                for button in pointer_state.buttons.iter() {
                    pending_input.events.push(egui::Event::PointerButton {
                        pos:        egui::Pos2 { x: location.0 as _, y: location.1 as _ },
                        button:     convert_button(*button),
                        pressed:    false,
                        modifiers:  pending_input.modifiers,
                    });
                }
            }
        }

        Pointer(_, _, _) => {
            // Other pointer actions (Enter, Leave, Drag, Cancel) are ignored
        }

        // Modifiers: key down
        KeyDown(_, Some(draw::Key::ModifierShift))  => { pending_input.modifiers.shift = true; },
        KeyDown(_, Some(draw::Key::ModifierAlt))    => { pending_input.modifiers.alt = true; },

        #[cfg(target_os="macos")]
        KeyDown(_, Some(draw::Key::ModifierMeta))   => { pending_input.modifiers.mac_cmd = true; pending_input.modifiers.command = true; },
        #[cfg(target_os="macos")]
        KeyDown(_, Some(draw::Key::ModifierCtrl))   => { pending_input.modifiers.ctrl = true; },

        #[cfg(not(target_os="macos"))]
        KeyDown(_, Some(draw::Key::ModifierMeta))   => { pending_input.modifiers.mac_cmd = true; },
        #[cfg(not(target_os="macos"))]
        KeyDown(_, Some(draw::Key::ModifierCtrl))   => { pending_input.modifiers.ctrl = true; pending_input.modifiers.command = true; },

        // Modifiers: key up
        KeyUp(_, Some(draw::Key::ModifierShift))    => { pending_input.modifiers.shift = false; },
        KeyUp(_, Some(draw::Key::ModifierAlt))      => { pending_input.modifiers.alt = false; },

        #[cfg(target_os="macos")]
        KeyUp(_, Some(draw::Key::ModifierMeta))     => { pending_input.modifiers.mac_cmd = false; pending_input.modifiers.command = false; },
        #[cfg(target_os="macos")]
        KeyUp(_, Some(draw::Key::ModifierCtrl))     => { pending_input.modifiers.ctrl = false; },

        #[cfg(not(target_os="macos"))]
        KeyUp(_, Some(draw::Key::ModifierMeta))     => { pending_input.modifiers.mac_cmd = false; },
        #[cfg(not(target_os="macos"))]
        KeyUp(_, Some(draw::Key::ModifierCtrl))     => { pending_input.modifiers.ctrl = false; pending_input.modifiers.command = false; },

        // Other key presses
        KeyDown(_, key) => {
            if let Some(key) = key.and_then(|key| convert_key(key)) {
                // Add a key down event
                // TODO: we can't currently track repeats
                pending_input.events.push(egui::Event::Key {
                    key:            key,
                    physical_key:   None,
                    pressed:        true,
                    repeat:         false,
                    modifiers:      pending_input.modifiers,
                });
            }
        }

        KeyUp(_, key) => {
            if let Some(key) = key.and_then(|key| convert_key(key)) {
                // Add a key up event
                // TODO: we can't currently track repeats
                pending_input.events.push(egui::Event::Key {
                    key:            key,
                    physical_key:   None,
                    pressed:        false,
                    repeat:         false,
                    modifiers:      pending_input.modifiers,
                });
            }
        }
    }
}

///
/// Writes out the instructions to fill a region
///
fn draw_fill(fill: &egui::Color32, drawing: &mut Vec<canvas::Draw>) {
    // Convert to rgba and do nothing if the colour is empty
    let rgba = egui::Rgba::from(*fill);
    if rgba.a() <= 0.0 { return; }

    // Fill with this colour
    drawing.fill_color(canvas::Color::Rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a()));
    drawing.fill();
}

///
/// Writes out the instructions to stroke a region
///
fn draw_stroke(stroke: &egui::Stroke, drawing: &mut Vec<canvas::Draw>) {
    // Do nothing if the width is < 0.0
    if stroke.width <= 0.0 { return; }

    // Convert the colour, and do nothing if it's empty
    let rgba = egui::Rgba::from(stroke.color);
    if rgba.a() <= 0.0 { return; }

    // Stroke with this width and colour
    drawing.line_width(stroke.width);
    drawing.stroke_color(canvas::Color::Rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a()));
    drawing.stroke();
}

///
/// Writes out the instructions to stroke a region
///
fn draw_path_stroke(stroke: &epaint::PathStroke, drawing: &mut Vec<canvas::Draw>) {
    // Do nothing if the width is < 0.0
    if stroke.width <= 0.0 { return; }

    // TODO: deal with StrokeKind (flo_draw doesn't natively support this)

    // Stroke with this width and colour
    match &stroke.color {
        epaint::ColorMode::Solid(color) => {
            let rgba = egui::Rgba::from(*color);
            if rgba.a() <= 0.0 { return; }

            drawing.line_width(stroke.width);
            drawing.stroke_color(canvas::Color::Rgba(rgba.r(), rgba.g(), rgba.b(), rgba.a()));
            drawing.stroke();
        }

        epaint::ColorMode::UV(callback) => {
            // TODO: flo_draw doesn't have Gouraud shading, so we can't really implement this
        }
    }
}

///
/// Draws a filled region using the specified brush
///
fn draw_fill_brush(brush: &epaint::Brush, drawing: &mut Vec<canvas::Draw>, region: ((f32, f32), (f32, f32))) {
    // Select the flo_draw texture to use (TODO: use separate namespaces for the user and managed textures?)
    let texture_id = match brush.fill_texture_id {
        epaint::TextureId::Managed(id)  => canvas::TextureId(id as _),
        epaint::TextureId::User(id)     => canvas::TextureId((id | 0x100000000) as _),
    };

    // The UVs go from 0-1 so we need to add/multiply the region as flo_draw works by setting the position of the texture on the canvas
    let mx = region.1.0 - region.0.0;
    let my = region.1.1 - region.0.1;

    let uv_min_x = (brush.uv.min.x + region.0.0) * mx;
    let uv_min_y = (brush.uv.min.y + region.0.1) * my;
    let uv_max_x = (brush.uv.max.x + region.0.0) * mx;
    let uv_max_y = (brush.uv.max.y + region.0.1) * my;

    drawing.fill_texture(texture_id, uv_min_x, uv_min_y, uv_max_x, uv_max_y);
    drawing.fill();
}

///
/// Draws a rectangle shape
///
fn draw_rect(rect_shape: &epaint::RectShape, drawing: &mut Vec<canvas::Draw>) {
    // Create the rectangle path
    // TODO: rounded corners
    // TODO: round to pixels if requested
    // TODO: deal with the stroke kind
    drawing.new_path();
    drawing.rect(rect_shape.rect.min.x, rect_shape.rect.min.y, rect_shape.rect.max.x, rect_shape.rect.max.y);

    // TODO: render to a sprite and blur if there's a blur width set
    // Fill with the requested fill colour
    draw_fill(&rect_shape.fill, drawing);
    draw_stroke(&rect_shape.stroke, drawing);

    // Render the texture if there is one
    if let Some(brush) = &rect_shape.brush {
        let bounds = ((rect_shape.rect.min.x, rect_shape.rect.min.y), (rect_shape.rect.max.x, rect_shape.rect.max.y));
        draw_fill_brush(&**brush, drawing, bounds);
    }
}

///
/// Draws a shape to a drawing vec
///
fn draw_shape(shape: &egui::Shape, drawing: &mut Vec<canvas::Draw>) {
    use canvas::{Draw, LayerId};
    use egui::{Shape};
    use Shape::*;

    match shape {
        Noop                            => { }
        Vec(shapes)                     => { shapes.iter().for_each(|shape| draw_shape(shape, drawing)); }
        Circle(circle)                  => { drawing.new_path(); drawing.circle(circle.center.x, circle.center.y, circle.radius); draw_fill(&circle.fill, drawing); draw_stroke(&circle.stroke, drawing); }
        Ellipse(_)                      => { /* TODO */ }
        LineSegment{points, stroke}     => { drawing.new_path(); drawing.move_to(points[0].x, points[0].y); points.iter().skip(1).for_each(|point| drawing.line_to(point.x, point.y)); draw_stroke(stroke, drawing); }
        Path(path_shape)                => { drawing.new_path(); drawing.move_to(path_shape.points[0].x, path_shape.points[0].y); path_shape.points.iter().skip(1).for_each(|point| drawing.line_to(point.x, point.y)); if path_shape.closed { drawing.close_path(); } draw_fill(&path_shape.fill, drawing); draw_path_stroke(&path_shape.stroke, drawing); }
        Rect(rect_shape)                => { draw_rect(rect_shape, drawing); }
        Text(text_shape)                => { }
        Mesh(mesh_shape)                => { }
        QuadraticBezier(quad_bezier)    => { }
        CubicBezier(cubic_bezier)       => { }
        Callback(_)                     => { /* Not supported */ }
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

    // Send the drawing if we generated any drawing instructions
    if drawing.len() > initial_len {
        drawing.extend([
            Draw::PopState,
        ]);

        drawing_target.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
    }
}

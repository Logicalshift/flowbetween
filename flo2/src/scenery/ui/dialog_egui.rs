use super::dialog::*;
use super::focus::*;

use flo_scene::*;
use flo_scene::programs::*;
use flo_draw as draw;
use flo_draw::canvas as canvas;
use flo_draw::canvas::scenery::*;

use egui;
use futures::prelude::*;

use std::sync::*;

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
/// Converts DrawEvents to egui events
///
fn convert_events(pending_input: &mut egui::RawInput, event: draw::DrawEvent) {
    use draw::DrawEvent::*;

    match event {
        Redraw              => { }
        NewFrame            => { }
        Scale(new_scale)    => { }  // TODO: think this affects egui's texture rendering
        Resize(_, _)        => { }
        CanvasTransform(_)  => { }
        Closed              => { }

        Pointer(action, pointer_id, pointer_state) => {

        }

        KeyDown(_, key) => {

        }

        KeyUp(_, key) => {

        }
    }
}

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
                // Not waiting for any more idle events
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

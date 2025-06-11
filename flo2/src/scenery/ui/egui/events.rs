use super::key::*;

use flo_draw as draw;
use egui;

///
/// Converts a button press from flo_draw to egui
///
pub fn convert_button(button: draw::Button) -> egui::PointerButton {
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
pub fn convert_events(pending_input: &mut egui::RawInput, event: draw::DrawEvent) {
    use draw::DrawEvent::*;
    use draw::PointerAction;

    match event {
        Redraw              => { }
        NewFrame            => { }
        Resize(_, _)        => { }
        CanvasTransform(_)  => { }
        Closed              => { }

        Scale(new_scale)    => {
            let mut viewport_info                   = egui::ViewportInfo::default();
            viewport_info.native_pixels_per_point   = Some(new_scale as _);

            pending_input.viewports.insert(egui::ViewportId::ROOT, viewport_info);
        }

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

use super::widget::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_event_parameter::*;

use gtk::prelude::*;
use gdk;

use std::rc::*;
use std::cell::*;

/// Maximum distance the mouse can move for a click to be considered a clicked
const MAX_DISTANCE: f64 = 5.0;

///
/// Provides actions for the 'click' action for Flo widgets
///
pub struct ClickActions {
    /// True if the button is pressed
    button_pressed: bool,

    /// Where the button was pressed
    button_press_location: (f64, f64)
}

impl ClickActions {
    pub fn wire_widget<W: GtkUiWidget>(event_sink: GtkEventSink, widget: &W, action_name: String) {
        use self::GtkEvent::Event;

        let widget_id   = widget.id();

        // The state is used to track where the button press starts and ends
        let state       = ClickActions {
            button_pressed:         false,
            button_press_location:  (0.0, 0.0)
        };
        let state       = Rc::new(RefCell::new(state));

        // For basic widgets with no explicit click action, we just detect the button press event
        widget.get_underlying().add_events(gdk::EventMask::BUTTON_PRESS_MASK);
        widget.get_underlying().add_events(gdk::EventMask::BUTTON_RELEASE_MASK);

        {
            let state = Rc::clone(&state);

            // On press: update the state to note that the button has been pressed
            widget.get_underlying()
                .connect_button_press_event(move |_, button| {
                    if button.get_state().is_empty() && button.get_button() == 1 {
                        // Left mouse button down with no modifiers = click
                        state.borrow_mut().button_pressed           = true;
                        state.borrow_mut().button_press_location    = button.get_position();
                        Inhibit(true)
                    } else if button.get_button() == 1 {
                        // Not a click but we stil want to inhibit other actions here
                        Inhibit(true)
                    } else {
                        // Other button down = continue with other event handlers
                        Inhibit(false)
                    }
                });
        }

        {
            let state = Rc::clone(&state);

            // On release: If the mouse hasn't moved MAX_DISTANCE, then fire the click event
            widget.get_underlying()
                .connect_button_release_event(move |_, button| {
                    let was_pressed = state.borrow().button_pressed;
                    let start_pos   = state.borrow().button_press_location;
                    let end_pos     = button.get_position();

                    let distance_x  = start_pos.0-end_pos.0;
                    let distance_y  = start_pos.1-end_pos.1;
                    let distance_sq = distance_x*distance_x + distance_y*distance_y;

                    state.borrow_mut().button_pressed = false;

                    if button.get_button() == 1 {
                        // Left mouse button released with no modifiers = click
                        if was_pressed && distance_sq <= MAX_DISTANCE * MAX_DISTANCE {
                            publish_event(&event_sink, Event(widget_id, action_name.clone(), GtkEventParameter::None));
                        }
                        Inhibit(true)
                    } else {
                        // Other button down = continue with other event handlers
                        Inhibit(false)
                    }
                });
        }
    }
}

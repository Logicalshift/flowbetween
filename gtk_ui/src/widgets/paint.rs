use super::widget::*;
use super::widget_data::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_widget_event_type::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Provides support for the painting events for a widget
/// 
pub struct PaintActions {
    /// Where the paint events should be sent to
    event_sink: GtkEventSink,

    /// True if we're following some paint events
    painting: bool
}

impl PaintActions {
    /// 
    /// Creates new paint data
    /// 
    fn new(event_sink: GtkEventSink) -> PaintActions {
        PaintActions {
            event_sink: event_sink,
            painting:   false
        }
    }

    ///
    /// Wires an existing widget for paint events
    /// 
    pub fn wire_widget<W: GtkUiWidget>(widget_data: &WidgetData, event_sink: RefCell<GtkEventSink>, widget: &W, device: GtkPaintDevice) {
        let widget_id       = widget.id();
        let existing_wiring = widget_data.get_widget_data::<PaintActions>(widget_id);

        println!("Wiring");

        match existing_wiring {
            Some(paint) => {
                // TODO: Add the device to the set already in use
            },

            None => {
                println!("New");

                // Create some new wiring
                widget_data.set_widget_data(widget_id, PaintActions::new(event_sink.into_inner()));

                // Fetch the wiring
                let new_wiring = widget_data.get_widget_data::<PaintActions>(widget_id).unwrap();

                // Connect the paint events to this widget
                Self::connect_events(widget.get_underlying(), Rc::clone(&*new_wiring));

                // TODO: add this device to the set supported by this widget
            }
        }
    }

    ///
    /// Connects paint events to a GTK widget
    /// 
    fn connect_events(widget: &gtk::Widget, paint: Rc<RefCell<PaintActions>>) {
        Self::connect_button_pressed(widget, Rc::clone(&paint));
        Self::connect_button_released(widget, Rc::clone(&paint));
        Self::connect_motion(widget, paint);
    }

    ///
    /// Sets up the button pressed event for a painting action
    /// 
    fn connect_button_pressed(widget: &gtk::Widget, paint: Rc<RefCell<PaintActions>>) {
        widget.connect_button_press_event(move |widget, event| {
            let mut paint = paint.borrow_mut();

            // TODO: check if this is a device we want to follow

            // Note that we're painting
            paint.painting = true;

            // TODO: track motion
            // TODO: generate the start event on the sink

            println!("Button down");

            // Prevent standard handling
            Inhibit(true)
        });
    }

    ///
    /// Sets up the button released event for a painting action
    /// 
    fn connect_button_released(widget: &gtk::Widget, paint: Rc<RefCell<PaintActions>>) {
        widget.connect_button_release_event(move |widget, event| {
            let mut paint = paint.borrow_mut();

            // TODO: check that the button being released is one on the device we're following
            // TODO: cancel touch events if the stylus is used instead

            if paint.painting {
                // Note that we're no longer painting
                paint.painting = false;

                println!("Button released");

                // TODO: stop tracking motion
                // TODO: generate the finished event on the sink

                // Painting: inhibit the usual behaviour
                Inhibit(true)
            } else {
                // Not painting: allow whatever other handling is present to take place
                Inhibit(false)
            }
        });
    }

    ///
    /// Sets up the motion event for a painting action
    /// 
    fn connect_motion(widget: &gtk::Widget, paint: Rc<RefCell<PaintActions>>) {
        widget.connect_motion_notify_event(move |widget, event| {
            let mut paint = paint.borrow_mut();

            if paint.painting {
                // TODO: also check that we're following the right device
                // TODO: send a motion event to the target
                println!("Motion");

                Inhibit(true)
            } else {
                println!("Also motion");

                Inhibit(false)
            }
        });
    }
}

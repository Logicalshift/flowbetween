use super::widget::*;
use super::widget_data::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_event_parameter::*;
use super::super::gtk_widget_event_type::*;

use glib::object::Downcast;
use glib::translate::*;
use gtk;
use gtk::prelude::*;
use gdk;
use gdk::prelude::*;
use gdk_sys;
use cairo;
use cairo::prelude::*;
use futures::*;

use std::rc::*;
use std::cell::*;
use std::mem;

///
/// Provides support for the painting events for a widget
/// 
pub struct PaintActions {
    /// The ID of the widget these actions are for
    widget_id: WidgetId,

    /// The name of the event to generate
    event_name: String,

    /// Where the paint events should be sent to
    event_sink: GtkEventSink,

    /// True if we're following some paint events
    painting: bool,

    /// The transformation matrix in use at the time the event started
    transform: cairo::Matrix
}

impl PaintActions {
    /// 
    /// Creates new paint data
    /// 
    fn new(widget_id: WidgetId, event_name: String, event_sink: GtkEventSink) -> PaintActions {
        PaintActions {
            widget_id:  widget_id,
            event_name: event_name,
            event_sink: event_sink,
            painting:   false,
            transform:  cairo::Matrix::identity()
        }
    }

    ///
    /// Wires an existing widget for paint events
    /// 
    pub fn wire_widget<W: GtkUiWidget>(widget_data: Rc<WidgetData>, event_sink: RefCell<GtkEventSink>, widget: &W, event_name: String, device: GtkPaintDevice) {
        let widget_id       = widget.id();
        let existing_wiring = widget_data.get_widget_data::<PaintActions>(widget_id);

        match existing_wiring {
            Some(paint) => {
                // TODO: Add the device to the set already in use
                // TODO: if the name is different from the original event name, we'll just use that
            },

            None => {
                // Create some new wiring
                widget_data.set_widget_data(widget_id, PaintActions::new(widget_id, event_name, event_sink.into_inner()));

                // Fetch the wiring
                let new_wiring = widget_data.get_widget_data::<PaintActions>(widget_id).unwrap();

                // Connect the paint events to this widget
                Self::connect_events(widget_data, widget.get_underlying(), widget.id(), Rc::clone(&*new_wiring));

                // TODO: add this device to the set supported by this widget
            }
        }
    }

    fn device_for_event(event: &gdk::Event) {
        let device: gdk::Device = unsafe {
            let sys_device: *const gdk_sys::GdkEvent = mem::transmute(event.as_ref());
            let sys_device = gdk_sys::gdk_event_get_source_device(sys_device);
            gdk::Device::from_glib_borrow(sys_device).downcast_unchecked()
        };

        println!("{:?}", device);
        println!("{:?} {:?} {:?} {:?}", device.get_device_type(), device.get_mode(), device.get_name(), device.get_source());
    }

    ///
    /// Connects paint events to a GTK widget
    /// 
    fn connect_events(widget_data: Rc<WidgetData>, widget: &gtk::Widget, widget_id: WidgetId, paint: Rc<RefCell<PaintActions>>) {
        // Make sure we're generating the appropriate events on this widget
        widget.add_events((gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK | gdk::EventMask::BUTTON_MOTION_MASK).bits() as i32);

        // Connect to the signals
        Self::connect_button_pressed(Rc::clone(&widget_data), widget, widget_id, Rc::clone(&paint));
        Self::connect_button_released(Rc::clone(&widget_data), widget, widget_id, Rc::clone(&paint));
        Self::connect_motion(widget_data, widget, widget_id, paint);
    }

    ///
    /// Sets up the button pressed event for a painting action
    /// 
    fn connect_button_pressed(widget_data: Rc<WidgetData>, widget: &gtk::Widget, widget_id: WidgetId, paint: Rc<RefCell<PaintActions>>) {
        widget.connect_button_press_event(move |widget, event| {
            let mut paint = paint.borrow_mut();

            // Create the painting data
            let widget_id       = paint.widget_id;
            let event_name      = paint.event_name.clone();
            let mut painting    = GtkPainting::from_button(event);

            paint.transform     = cairo::Matrix::identity();

            if let Some(transform) = widget_data.get_widget_data(widget_id) {
                paint.transform = *transform.borrow();
            }

            painting.transform(&paint.transform);

            // TODO: check if this is a device we want to follow

            // Note that we're painting
            paint.painting = true;

            // Generate the start event on the sink
            paint.event_sink.start_send(GtkEvent::Event(widget_id, event_name, GtkEventParameter::PaintStart(painting))).unwrap();

            // Prevent standard handling
            Inhibit(true)
        });
    }

    ///
    /// Sets up the button released event for a painting action
    /// 
    fn connect_button_released(widget_data: Rc<WidgetData>, widget: &gtk::Widget, widget_id: WidgetId, paint: Rc<RefCell<PaintActions>>) {
        widget.connect_button_release_event(move |widget, event| {
            let mut paint = paint.borrow_mut();

            // TODO: check that the button being released is one on the device we're following
            // TODO: cancel touch events if the stylus is used instead

            if paint.painting {
                // Note that we're no longer painting
                paint.painting = false;

                // Create the painting data
                let widget_id       = paint.widget_id;
                let event_name      = paint.event_name.clone();
                let mut painting    = GtkPainting::from_button(event);

                painting.transform(&paint.transform);

                // Generate the start event on the sink
                paint.event_sink.start_send(GtkEvent::Event(widget_id, event_name, GtkEventParameter::PaintFinish(painting))).unwrap();

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
    fn connect_motion(widget_data: Rc<WidgetData>, widget: &gtk::Widget, widget_id: WidgetId, paint: Rc<RefCell<PaintActions>>) {
        // Searches the widget hierarchy to find the widget with a window and disables event compression on it
        fn disable_compression(widget: &gtk::Widget) {
            if let Some(window) = widget.get_window() {
                window.set_event_compression(false);
            } else if let Some(parent) = widget.get_parent() {
                disable_compression(&parent);
            }
        }

        // If the widget is already realized then disable compression
        disable_compression(widget);

        // Disable compression if the widget is realized
        widget.connect_realize(move |widget| {
            disable_compression(widget);
        });

        widget.connect_motion_notify_event(move |widget, event| {
            let mut paint = paint.borrow_mut();

            if paint.painting {
                // Create the painting data
                let widget_id       = paint.widget_id;
                let event_name      = paint.event_name.clone();
                let mut painting    = GtkPainting::from_motion(event);

                painting.transform(&paint.transform);

                // Generate the start event on the sink
                paint.event_sink.start_send(GtkEvent::Event(widget_id, event_name, GtkEventParameter::PaintContinue(painting))).unwrap();

                // TODO: also check that we're following the right device

                // Request more motions
                unsafe { gdk_sys::gdk_event_request_motions(event.as_ref()); }

                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });
    }
}

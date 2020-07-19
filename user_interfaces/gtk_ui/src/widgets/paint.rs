use super::events::*;
use super::widget::*;
use super::widget_data::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_event_parameter::*;
use super::super::gtk_widget_event_type::*;

use gtk;
use gtk::prelude::*;
use gdk;
use gdk::prelude::*;
use gdk_sys;
use cairo;

use std::rc::*;
use std::cell::*;
use std::collections::HashSet;

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

    /// The transformation matrix in use at the time the event started
    transform: cairo::Matrix,

    /// True if the first motion should generate a start event
    need_start: bool,

    /// The input sources that should generate these actions
    input_sources: HashSet<gdk::InputSource>,

    /// For mouse events, the buttons that we should respond to
    buttons: HashSet<u32>,

    /// The device that is currently being tracked
    active_device: Option<gdk::InputSource>
}

impl PaintActions {
    ///
    /// Creates new paint data
    ///
    fn new(widget_id: WidgetId, event_name: String, event_sink: GtkEventSink) -> PaintActions {
        PaintActions {
            widget_id:      widget_id,
            event_name:     event_name,
            event_sink:     event_sink,
            transform:      cairo::Matrix::identity(),
            need_start:     false,
            input_sources:  HashSet::new(),
            buttons:        HashSet::new(),
            active_device:  None
        }
    }

    ///
    /// Wires an existing widget for paint events
    ///
    pub fn wire_widget<W: GtkUiWidget>(widget_data: Rc<WidgetData>, event_sink: GtkEventSink, widget: &W, event_name: String, device: GtkPaintDevice) {
        let widget_id       = widget.id();
        let existing_wiring = widget_data.get_widget_data::<PaintActions>(widget_id);

        match existing_wiring {
            Some(paint) => {
                let mut paint = paint.borrow_mut();

                // Add the device to the set already in use
                paint.input_sources.extend(Vec::<gdk::InputSource>::from(device).into_iter());
                paint.buttons.extend(device.buttons().into_iter());

                // TODO: if the name is different from the original event name, we'll just use that
            },

            None => {
                // Create some new wiring
                widget_data.set_widget_data(widget_id, PaintActions::new(widget_id, event_name, event_sink));

                // Fetch the wiring
                let new_wiring = widget_data.get_widget_data::<PaintActions>(widget_id).unwrap();

                // Connect the paint events to this widget
                Self::connect_events(widget_data, widget.get_underlying(), Rc::clone(&*new_wiring));

                // Add this device to the set supported by this widget
                let mut new_wiring = new_wiring.borrow_mut();
                new_wiring.input_sources.extend(Vec::<gdk::InputSource>::from(device).into_iter());
                new_wiring.buttons.extend(device.buttons().into_iter());
            }
        }
    }

    ///
    /// Connects paint events to a GTK widget
    ///
    fn connect_events(widget_data: Rc<WidgetData>, widget: &gtk::Widget, paint: Rc<RefCell<PaintActions>>) {
        // Make sure we're generating the appropriate events on this widget
        widget.add_events(gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK | gdk::EventMask::BUTTON_MOTION_MASK);

        // Connect to the signals
        Self::connect_button_pressed(Rc::clone(&widget_data), widget, Rc::clone(&paint));
        Self::connect_button_released(widget, Rc::clone(&paint));
        Self::connect_motion(widget, paint);
    }

    ///
    /// Sets up the button pressed event for a painting action
    ///
    fn connect_button_pressed(widget_data: Rc<WidgetData>, widget: &gtk::Widget, paint: Rc<RefCell<PaintActions>>) {
        widget.connect_button_press_event(move |_widget, event| {
            let mut paint   = paint.borrow_mut();
            let device      = device_for_event(event);
            let source      = device.get_source();

            // Start tracking if the device for the button press is registered
            if (source == gdk::InputSource::Mouse || source == gdk::InputSource::Touchpad) && !paint.buttons.contains(&event.get_button()) {
                Inhibit(false)
            } else if paint.active_device.is_some() && source == gdk::InputSource::Touchscreen {
                // If the user is using any device other than touch already, never switch to the touch device
                Inhibit(false)
            } else if paint.input_sources.contains(&source) {
                // Create the painting data
                let widget_id       = paint.widget_id;
                let event_name      = paint.event_name.clone();
                let mut painting    = GtkPainting::from_button(event);

                paint.transform     = cairo::Matrix::identity();

                if let Some(transform) = widget_data.get_widget_data(widget_id) {
                    paint.transform = *transform.borrow();
                }

                painting.transform(&paint.transform);

                // Cancel any on-going paint operation (so we replace it with the current device)
                if let Some(current_device) = paint.active_device {
                    let current_device = paint_device_for_source(current_device);
                    publish_event(&paint.event_sink, GtkEvent::Event(widget_id, event_name.clone(), GtkEventParameter::PaintCancel(current_device)));
                }

                // Note that we're painting
                paint.active_device = Some(source);

                // Generate the start event on the sink
                if source != gdk::InputSource::Pen && source != gdk::InputSource::Eraser {
                    paint.need_start = false;
                    publish_event(&paint.event_sink, GtkEvent::Event(widget_id, event_name, GtkEventParameter::PaintStart(painting)));
                } else {
                    // For some reason, the button press event for styluses on Linux is often in the wrong place, so we skip the initial event (making it the motion event instead)
                    paint.need_start = true;
                }

                // Prevent standard handling
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });
    }

    ///
    /// Sets up the button released event for a painting action
    ///
    fn connect_button_released(widget: &gtk::Widget, paint: Rc<RefCell<PaintActions>>) {
        widget.connect_button_release_event(move |_widget, event| {
            let mut paint   = paint.borrow_mut();
            let device      = device_for_event(event);

            if paint.active_device == Some(device.get_source()) {
                // Note that we're no longer painting
                paint.active_device = None;

                // Create the painting data
                let widget_id       = paint.widget_id;
                let event_name      = paint.event_name.clone();
                let mut painting    = GtkPainting::from_button(event);

                painting.transform(&paint.transform);

                // Generate the start event on the sink
                publish_event(&paint.event_sink, GtkEvent::Event(widget_id, event_name, GtkEventParameter::PaintFinish(painting)));

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

        widget.connect_motion_notify_event(move |_widget, event| {
            let mut paint   = paint.borrow_mut();
            let device      = device_for_event(event);

            if paint.active_device == Some(device.get_source()) {
                // Create the painting data
                let widget_id       = paint.widget_id;
                let event_name      = paint.event_name.clone();
                let mut painting    = GtkPainting::from_motion(event);

                painting.transform(&paint.transform);

                // Generate the start event on the sink
                if paint.need_start {
                    paint.need_start = false;
                    publish_event(&paint.event_sink, GtkEvent::Event(widget_id, event_name, GtkEventParameter::PaintStart(painting)));
                } else {
                    publish_event(&paint.event_sink, GtkEvent::Event(widget_id, event_name, GtkEventParameter::PaintContinue(painting)));
                }

                // Request more motions
                unsafe { gdk_sys::gdk_event_request_motions(event.as_ref()); }

                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });
    }
}

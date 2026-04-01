use super::widget::*;
use super::widget_data::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_event_parameter::*;

use gtk;
use gtk::prelude::*;
use gdk;

use std::rc::*;
use std::cell::*;

///
/// Provides the implementation of the 'drag' action for Flo widgets
///
pub struct DragActions {
    /// Where events for these actions should be sent
    event_sink: GtkEventSink,

    /// True if we're dragging the widget
    dragging: bool,

    /// Where the drag started in the widget
    start_point: (f64, f64),

    /// Names of the events to generate for this widget
    event_names: Vec<String>
}

impl DragActions {
    ///
    /// Creates a new drag actions object
    ///
    fn new(event_sink: GtkEventSink) -> DragActions {
        DragActions {
            event_sink:     event_sink,
            dragging:       false,
            start_point:    (0.0, 0.0),
            event_names:    vec![]
        }
    }

    ///
    /// Wires a widget up for the drag action
    ///
    pub fn wire_widget<W: GtkUiWidget>(widget_data: Rc<WidgetData>, event_sink: GtkEventSink, widget: &W, event_name: String) {
        let widget_id   = widget.id();
        let drag_wiring = widget_data.get_widget_data::<DragActions>(widget_id);

        match drag_wiring {
            Some(existing_wiring) => {
                // Drag actions are already attached to this widget: just add new event names
                existing_wiring.borrow_mut().event_names.push(event_name)
            },

            None => {
                // Create some new wiring
                let mut drag_wiring = Self::new(event_sink);
                drag_wiring.event_names.push(event_name);

                widget_data.set_widget_data(widget_id, drag_wiring);

                // Connect events
                let drag_wiring = widget_data.get_widget_data::<DragActions>(widget_id).unwrap();
                Self::connect_events(widget.get_underlying(), widget.id(), Rc::clone(&*drag_wiring));
            }
        }
    }

    ///
    /// Returns the drag position (in the main window) for the specified mouse position
    ///
    fn drag_position_for_position(widget: &gtk::Widget, position: (f64, f64)) -> (f64, f64) {
        let parent      = widget.get_toplevel().unwrap();

        let position    = (position.0 as i32, position.1 as i32);
        let position    = widget.translate_coordinates(&parent, position.0, position.1).unwrap();

        (position.0 as f64, position.1 as f64)
    }

    ///
    /// Connects the events for a drag actions object
    ///
    fn connect_events(widget: &gtk::Widget, widget_id: WidgetId, drag_actions: Rc<RefCell<Self>>) {
        // Request the events
        widget.add_events(gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK | gdk::EventMask::BUTTON_MOTION_MASK);

        // Connect the signals
        Self::connect_press(widget, widget_id, Rc::clone(&drag_actions));
        Self::connect_motion(widget, widget_id, Rc::clone(&drag_actions));
        Self::connect_release(widget, widget_id, Rc::clone(&drag_actions));
    }

    ///
    /// Responds to the user pressing a button over the draggable widget
    ///
    fn connect_press(widget: &gtk::Widget, widget_id: WidgetId, drag_actions: Rc<RefCell<Self>>) {
        widget.connect_button_press_event(move |widget, button| {
            let mut drag_actions    = drag_actions.borrow_mut();

            if !drag_actions.dragging {
                // Start dragging
                let position = Self::drag_position_for_position(widget, button.get_position());

                drag_actions.dragging       = true;
                drag_actions.start_point    = position;

                // Send the start event
                let event_names = drag_actions.event_names.clone();
                let event_sink  = &drag_actions.event_sink;

                event_names.into_iter().for_each(|name| {
                    publish_event(event_sink, GtkEvent::Event(widget_id, name, GtkEventParameter::DragStart(position.0, position.1)));
                });

                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });
    }

    ///
    /// Responds to the user dragging the widget
    ///
    fn connect_motion(widget: &gtk::Widget, widget_id: WidgetId, drag_actions: Rc<RefCell<Self>>) {
        widget.connect_motion_notify_event(move |widget, button| {
            let drag_actions    = drag_actions.borrow_mut();

            if drag_actions.dragging {
                // Continue dragging
                let position = Self::drag_position_for_position(widget, button.get_position());

                // Send the start event
                let start_point = drag_actions.start_point;
                let event_names = drag_actions.event_names.clone();
                let event_sink  = &drag_actions.event_sink;

                event_names.into_iter().for_each(|name| {
                    publish_event(event_sink, GtkEvent::Event(widget_id, name, GtkEventParameter::DragContinue(start_point, position)));
                });

                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });
    }

    ///
    /// Responds to the user releasing the button while dragging the widget
    ///
    fn connect_release(widget: &gtk::Widget, widget_id: WidgetId, drag_actions: Rc<RefCell<Self>>) {
        widget.connect_button_release_event(move |widget, button| {
            let mut drag_actions    = drag_actions.borrow_mut();

            if drag_actions.dragging {
                // Finish dragging
                drag_actions.dragging = false;

                let position = Self::drag_position_for_position(widget, button.get_position());

                // Send the start event
                let start_point = drag_actions.start_point;
                let event_names = drag_actions.event_names.clone();
                let event_sink  = &drag_actions.event_sink;

                event_names.into_iter().for_each(|name| {
                    publish_event(&event_sink, GtkEvent::Event(widget_id, name, GtkEventParameter::DragContinue(start_point, position)));
                });

                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });
    }
}

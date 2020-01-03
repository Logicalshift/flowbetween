use super::image::*;
use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_widget_event_type::*;
use super::super::gtk_event_parameter::*;

use flo_ui::*;

use glib::prelude::*;
use gtk;
use gtk::prelude::*;
use gdk;
use gdk::prelude::*;
use gdk_pixbuf;

use std::ops::Range;
use std::rc::*;
use std::cell::*;
use std::f64;

struct RotorData {
    /// The image displayed in the rotor (or none for no image)
    image: Option<gdk_pixbuf::Pixbuf>,

    /// Child widgets (manually drawn)
    child_widgets: Vec<Rc<RefCell<dyn GtkUiWidget>>>,

    /// Value the rotor is set to
    value: f64,

    /// Range of values accepted by this rotor
    range: Range<f64>,

    /// Whether or not the rotor is currently being dragged
    dragging: bool,

    /// Angle where dragging started
    initial_angle: f64,

    /// Value when dragging started
    initial_value: f64,

    /// Event names and sinks for set events
    set_events: Vec<(String, GtkEventSink)>,

    /// Event names and sinks for edit events
    edit_events: Vec<(String, GtkEventSink)>
}

///
/// The rotor widget is essentially a scale widget that shows its value by rotating its contents
///
/// GTK doesn't support rotation of arbitrary widgets, so this currently fakes it by manually calling the
/// draw routine (which means that children of this widget are generally supposed to be drawing areas)
///
pub struct FloRotorWidget {
    id: WidgetId,

    /// The widget (a drawing area)
    widget: gtk::Widget,

    /// Data that's shared with the event handlers for this widget
    data: Rc<RefCell<RotorData>>
}

impl FloRotorWidget {
    ///
    /// Creates a new rotor control
    ///
    pub fn new<W: Clone+Cast+IsA<gtk::Widget>+IsA<gtk::DrawingArea>>(id: WidgetId, widget: W) -> FloRotorWidget {
        // Create the data
        let data = RotorData {
            image:          None,
            child_widgets:  vec![],
            value:          0.0,
            range:          0.0..1.0,
            dragging:       false,
            initial_angle:  0.0,
            initial_value:  0.0,
            set_events:     vec![],
            edit_events:    vec![]
        };
        let data = Rc::new(RefCell::new(data));

        // Register events
        Self::connect_signals(id, widget.clone().upcast::<gtk::Widget>(), Rc::clone(&data));

        // Generate the final widget
        FloRotorWidget {
            id:     id,
            widget: widget.upcast::<gtk::Widget>(),
            data:   data
        }
    }

    ///
    /// Wires up the signals for this rotor widget
    ///
    fn connect_signals(widget_id: WidgetId, widget: gtk::Widget, data: Rc<RefCell<RotorData>>) {
        Self::connect_drawing(&widget, Rc::clone(&data));
        Self::connect_size_allocate(&widget, Rc::clone(&data));
        Self::connect_drag(widget_id, &widget, Rc::clone(&data));
    }

    ///
    /// Connects the button press, release and motion events
    ///
    fn connect_drag(widget_id: WidgetId, widget: &gtk::Widget, data: Rc<RefCell<RotorData>>) {
        // Want the events for the various buttons and drags etc
        widget.add_events(gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::BUTTON_RELEASE_MASK | gdk::EventMask::BUTTON_MOTION_MASK);

        // Start dragging when the user presses the left mouse button
        {
            let data = data.clone();
            widget.connect_button_press_event(move |widget, button| {
                if button.get_button() == 1 {
                    // Start dragging the rotor
                    let mut data        = data.borrow_mut();
                    let (x, y)          = button.get_position();

                    data.dragging       = true;
                    data.initial_angle  = Self::angle_for_point(widget, x, y);
                    data.initial_value  = data.value;

                    // Prevent default handling
                    Inhibit(true)
                } else {
                    // Other buttons are passed through
                    Inhibit(false)
                }
            });
        }

        // Stop dragging when the user releases the mouse button
        {
            let data = data.clone();
            widget.connect_button_release_event(move |_widget, _button| {
                let mut data = data.borrow_mut();

                if data.dragging {
                    // No longer dragging
                    data.dragging = false;

                    // Send set events
                    let value = data.value as f64;
                    data.set_events.iter_mut().for_each(|&mut (ref event_name, ref mut sink)| {
                        publish_event(sink, GtkEvent::Event(widget_id, event_name.clone(), GtkEventParameter::ScaleValue(value)));
                    });

                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            });
        }

        // Change the value as the user drags the rotor
        {
            let data        = data.clone();
            let widget_id   = widget_id;
            widget.connect_motion_notify_event(move |widget, motion| {
                let mut data = data.borrow_mut();

                if data.dragging {
                    // Fetch the current angle
                    let (x, y)              = motion.get_position();
                    let current_angle       = Self::angle_for_point(widget, x, y);

                    // New value depends on the angle difference
                    let range               = (data.range.end - data.range.start).max(0.01);
                    let angle_difference    = current_angle - data.initial_angle;
                    let value_difference    = range * (angle_difference/360.0);

                    data.value              = data.initial_value + value_difference;
                    data.value              = ((data.value - data.range.start) % range) + data.range.start;
                    if data.value < data.range.start {
                        data.value += range;
                    }

                    // Send edit events
                    let value = data.value as f64;
                    data.edit_events.iter_mut().for_each(|&mut (ref event_name, ref mut sink)| {
                        publish_event(sink, GtkEvent::Event(widget_id, event_name.clone(), GtkEventParameter::ScaleValue(value)));
                    });

                    // Redraw the widget with the new value
                    widget.queue_draw();

                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            });
        }
    }

    ///
    /// Handles resizing the widget
    ///
    fn connect_size_allocate(widget: &gtk::Widget, data: Rc<RefCell<RotorData>>) {
        widget.connect_size_allocate(move |widget, allocation| {
            let mut allocation = *allocation;
            allocation.x = 0;
            allocation.y = 0;

            // Reallocate the size of all the child widgets
            data.borrow().child_widgets.iter()
                .for_each(|child_widget| {
                    child_widget.borrow().get_underlying().size_allocate(&mut allocation.clone());
                });

            // Redraw this widget
            widget.queue_draw();
        });
    }

    ///
    /// Connects the drawing event for a rotor widget
    ///
    fn connect_drawing(widget: &gtk::Widget, data: Rc<RefCell<RotorData>>) {
        widget.connect_draw(move |widget, context| {
            let data        = data.borrow();
            let allocation  = widget.get_allocation();

            context.save();

            // Rotate to the angle specified by the value
            context.translate((allocation.width as f64)/2.0, (allocation.height as f64)/2.0);

            let range = (data.range.end - data.range.start).max(0.01);
            let angle = 2.0 * f64::consts::PI * ((data.value-data.range.start)/(range));

            context.rotate(angle as f64);
            context.translate(-(allocation.width as f64)/2.0, -(allocation.height as f64)/2.0);

            // Draw the image, if there is one
            if let Some(ref image) = data.image.as_ref() {
                context.save();

                // Scale the image to fit
                let draw_width  = allocation.width as f64;
                let draw_height = allocation.height as f64;
                let img_width   = image.get_width() as f64;
                let img_height  = image.get_height() as f64;
                let scale   = (draw_width/img_width).min(draw_height/img_height);

                context.scale(scale, scale);

                // Paint the image
                context.set_source_pixbuf(image, 0.0, 0.0);
                context.paint();

                context.restore();
            }

            // Send drawing signals to any child widgets (they'll draw at the rotated angle)
            data.child_widgets.iter().for_each(|widget| {
                widget.borrow().draw_manual(context);
            });

            // Reset the context to its original settings
            context.restore();

            Inhibit(true)
        });
    }

    ///
    /// Returns the angle of a point in a rotor widget, in degrees
    ///
    fn angle_for_point(widget: &gtk::Widget, x: f64, y: f64) -> f64 {
        let allocation  = widget.get_allocation();
        let width       = allocation.width as f64;
        let height      = allocation.height as f64;

        // Assume that the node is a circle around its center
        let radius = width/2.0;

        let x = x - width/2.0;
        let y = y - height/2.0;

        if (x*x + y*y) < (radius*radius) {
            // If the point is within the main radius, then the angle is just the angle relative to the center
            f64::atan2(y, x) / (2.0*f64::consts::PI) * 360.0
        } else {
            // Really want to project a line onto the circle, then make the
            // extra angle be the distance from the rotor. This has a
            // similar effect but isn't quite as accurate.
            let angle               = f64::atan2(y, x) / (2.0*f64::consts::PI) * 360.0;
            let circumference       = f64::consts::PI*2.0*radius;
            let mut extra_distance  = -x;
            if x < -radius {
                extra_distance -= radius;
            } else if x > radius {
                extra_distance += radius;
            } else {
                extra_distance = 0.0;
            }

            angle + ((extra_distance/circumference)*360.0)
        }
    }
}


impl GtkUiWidget for FloRotorWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;
        use self::WidgetState::*;
        use self::Appearance::Image;
        use self::GtkWidgetEventType::{EditValue, SetValue};

        let being_dragged = self.data.borrow().dragging;

        match action {
            &State(SetValueFloat(value))                => { if !being_dragged { self.data.borrow_mut().value = value; self.widget.queue_draw(); } },
            &State(SetRangeMin(min_value))              => { self.data.borrow_mut().range.start = min_value; self.widget.queue_draw(); },
            &State(SetRangeMax(max_value))              => { self.data.borrow_mut().range.end = max_value; self.widget.queue_draw(); },

            &Appearance(Image(ref image_data)) => {
                // Update the image
                self.data.borrow_mut().image = Some(pixbuf_from_image(image_data.clone()));

                // Redraw the widget
                self.widget.queue_draw()
            },

            &RequestEvent(SetValue, ref event_name)     => { self.data.borrow_mut().set_events.push((event_name.clone(), flo_gtk.get_event_sink())); },
            &RequestEvent(EditValue, ref event_name)    => { self.data.borrow_mut().edit_events.push((event_name.clone(), flo_gtk.get_event_sink())); },

            other_action                                => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // Child widgets are drawn manually
        let mut allocation = self.widget.get_allocation();
        allocation.x = 0;
        allocation.y = 0;

        // They get the same allocation as this widget
        if allocation.width > 0 && allocation.height > 0 {
            children.iter().for_each(|child| child.borrow().get_underlying().size_allocate(&mut allocation.clone()));
        }

        // ... and are stored in the data structure
        self.data.borrow_mut().child_widgets = children;

        // Redraw the widget once done
        self.widget.queue_draw();
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.widget
    }

}

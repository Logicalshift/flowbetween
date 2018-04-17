use super::image::*;
use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_widget_event_type::*;

use flo_ui::*;

use glib::prelude::*;
use gtk;
use gtk::prelude::*;
use gdk::prelude::*;
use gdk_pixbuf;
use gdk_pixbuf::prelude::*;

use std::ops::Range;
use std::rc::*;
use std::cell::*;

struct RotorData {
    /// The image displayed in the rotor (or none for no image)
    image: Option<gdk_pixbuf::Pixbuf>,

    /// Child widgets (manually drawn)
    child_widgets: Vec<Rc<RefCell<GtkUiWidget>>>,
    
    /// Value the rotor is set to
    value: f32,

    /// Range of values accepted by this rotor
    range: Range<f32>
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
            range:          0.0..1.0
        };
        let data = Rc::new(RefCell::new(data));

        // Register events
        Self::connect_signals(widget.clone().upcast::<gtk::Widget>(), Rc::clone(&data));

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
    fn connect_signals(widget: gtk::Widget, data: Rc<RefCell<RotorData>>) {
        Self::connect_drawing(&widget, Rc::clone(&data));
        Self::connect_size_allocate(&widget, Rc::clone(&data));
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
            let angle = 360.0 * ((data.value-data.range.start)/(range));

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

        match action {
            &State(SetValueFloat(value))                => { self.data.borrow_mut().value = value; self.widget.queue_draw(); },
            &State(SetRangeMin(min_value))              => { self.data.borrow_mut().range.start = min_value; self.widget.queue_draw(); },
            &State(SetRangeMax(max_value))              => { self.data.borrow_mut().range.end = max_value; self.widget.queue_draw(); },

            &Appearance(Image(ref image_data)) => {
                // Update the image
                self.data.borrow_mut().image = Some(pixbuf_from_image(image_data.clone()));

                // Redraw the widget
                self.widget.queue_draw()
            },

            other_action                                => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
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

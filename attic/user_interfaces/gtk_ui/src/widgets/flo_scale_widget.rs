use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_event_parameter::*;
use super::super::gtk_widget_event_type::*;

use gtk;
use gtk::prelude::*;

use std::cell::*;
use std::rc::*;

///
/// Provides support for the scale widget
///
pub struct FloScaleWidget {
    /// The currently set min value
    min: f64,

    /// The currently set max value
    max: f64,

    /// The ID of the widget
    id: WidgetId,

    /// The scale's slider
    scale: gtk::Scale,

    /// The label as a widget
    widget: gtk::Widget,

    /// Flag that indicates if the user is pressing a mouse button (ie, dragging the scale)
    button_pressed: Rc<RefCell<bool>>
}

impl FloScaleWidget {
    ///
    /// Creates a new scale widget
    ///
    pub fn new(id: WidgetId, scale: gtk::Scale) -> FloScaleWidget {
        let button_pressed = Rc::new(RefCell::new(false));

        Self::connect_button_events(&scale, Rc::clone(&button_pressed));

        FloScaleWidget {
            min:            0.0,
            max:            0.0,
            id:             id,
            widget:         scale.clone().upcast::<gtk::Widget>(),
            scale:          scale,
            button_pressed: button_pressed
        }
    }

    ///
    /// Hooks up the button pressed event
    ///
    fn connect_button_events(scale: &gtk::Scale, button_pressed: Rc<RefCell<bool>>) {
        {
            // Set the button pressed flag when the user clicks the mouse over the scale
            let button_pressed = Rc::clone(&button_pressed);
            scale.connect_button_press_event(move |_, _| {
                *button_pressed.borrow_mut() = true;
                Inhibit(false)
            });
        }

        {
            // Clear the button pressed flag when the user releases the mouse over the scale
            let button_pressed = Rc::clone(&button_pressed);
            scale.connect_button_release_event(move |_, _| {
                *button_pressed.borrow_mut() = false;
                Inhibit(false)
            });
        }
    }
}

impl GtkUiWidget for FloScaleWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;
        use self::WidgetState::*;
        use self::GtkWidgetEventType::{EditValue, SetValue};

        match action {
            &State(SetValueFloat(value))                => self.scale.set_value(value as f64),
            &State(SetValueInt(value))                  => self.scale.set_value(value as f64),
            &State(SetRangeMin(min_value))              => {
                self.min = min_value as f64;
                self.scale.set_range(self.min.min(self.max), self.max.max(self.min));
            },
            &State(SetRangeMax(max_value))              => {
                self.max = max_value as f64;
                self.scale.set_range(self.min.min(self.max), self.max.max(self.min));
            },

            &RequestEvent(SetValue, ref event_name_ref)     => {
                // Set events are value changes that occur while the mouse button has been released
                let id              = self.id;
                let sink            = flo_gtk.get_event_sink();
                let event_name      = event_name_ref.clone();
                let button_pressed  = Rc::clone(&self.button_pressed);

                self.scale.connect_value_changed(move |widget| {
                    // Generate when the value is changed and the button is not held down
                    if *button_pressed.borrow() == false {
                        let new_value       = widget.get_value();

                        publish_event(&sink, GtkEvent::Event(id, event_name.clone(), GtkEventParameter::ScaleValue(new_value)));
                    }
                });

                let sink            = flo_gtk.get_event_sink();
                let event_name      = event_name_ref.clone();
                self.scale.connect_button_release_event(move |widget, _| {
                    // Also generate if the button is released
                    let new_value       = widget.get_value();
                    publish_event(&sink, GtkEvent::Event(id, event_name.clone(), GtkEventParameter::ScaleValue(new_value)));
                    Inhibit(false)
                });
            },

            &RequestEvent(EditValue, ref event_name)    => {
                // Edit events are value changes that occur while the mouse button is held down
                let last_value      = RefCell::new(self.scale.get_value());
                let id              = self.id;
                let sink            = flo_gtk.get_event_sink();
                let event_name      = event_name.clone();
                let button_pressed  = Rc::clone(&self.button_pressed);

                self.scale.connect_value_changed(move |widget| {
                    if *button_pressed.borrow() == true {
                        let new_value       = widget.get_value();
                        let mut last_value  = last_value.borrow_mut();

                        if new_value != *last_value {
                            *last_value = new_value;
                            publish_event(&sink, GtkEvent::Event(id, event_name.clone(), GtkEventParameter::ScaleValue(new_value)));
                        }
                    }
                });
            },

            other_action                                => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // Scales cannot have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.widget
    }
}

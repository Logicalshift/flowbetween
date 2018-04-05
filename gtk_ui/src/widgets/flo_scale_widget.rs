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
    widget: gtk::Widget
}

impl FloScaleWidget {
    ///
    /// Creates a new scale widget
    /// 
    pub fn new(id: WidgetId, scale: gtk::Scale) -> FloScaleWidget {
        FloScaleWidget {
            min:    0.0,
            max:    0.0,
            id:     id,
            widget: scale.clone().upcast::<gtk::Widget>(),
            scale:  scale,
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
            &State(SetRangeMin(min_value))              => {
                self.min = min_value as f64;
                self.scale.set_range(self.min.min(self.max), self.max.max(self.min));
            },
            &State(SetRangeMax(max_value))              => {
                self.max = max_value as f64;
                self.scale.set_range(self.min.min(self.max), self.max.max(self.min));
            },

            &RequestEvent(SetValue, ref event_name)     |
            &RequestEvent(EditValue, ref event_name)    => {
                // Right now, there doesn't seem to be an easy way to distinguish between values set during a drag and at the end
                // so we consider 'set' and 'edit' to be the same actions for now.
                let last_value  = RefCell::new(self.scale.get_value());
                let id          = self.id;
                let sink        = RefCell::new(flo_gtk.get_event_sink());
                let event_name  = event_name.clone();
                self.scale.connect_value_changed(move |widget| { 
                    let new_value       = widget.get_value();
                    let mut last_value  = last_value.borrow_mut();

                    if new_value != *last_value {
                        *last_value = new_value;
                    sink.borrow_mut().start_send(GtkEvent::Event(id, event_name.clone(), GtkEventParameter::ScaleValue(new_value))).unwrap();
                    }
                });
            },

            other_action                                => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // Scales cannot have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.widget
    }

}

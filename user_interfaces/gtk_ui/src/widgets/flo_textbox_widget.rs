use super::basic_widget::*;
use super::super::widgets::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_event_parameter::*;
use super::super::gtk_widget_event_type::*;

use gtk;
use gtk::prelude::*;
use futures::Sink;

use std::rc::*;
use std::cell::*;

///
/// Implements behaviour for the textbox (entry) widget
///
pub struct FloTextBoxWidget {
    /// The ID of this widget
    id: WidgetId,

    /// The entry widget
    widget: gtk::Entry,

    /// The entry again, but cast to a widget
    as_widget: gtk::Widget,
}

impl FloTextBoxWidget {
    ///
    /// Creates a new textbox widget
    ///
    pub fn new<W: Clone+Cast+IsA<gtk::Entry>+IsA<gtk::Widget>>(id: WidgetId, entry: W) -> FloTextBoxWidget {
        let entry = entry.upcast::<gtk::Entry>();

        entry.set_has_frame(false);
        entry.set_max_length(1024);
        entry.set_editable(true);
        entry.set_can_focus(true);

        FloTextBoxWidget {
            id:             id,
            widget:         entry.clone(),
            as_widget:      entry.clone().upcast::<gtk::Widget>()
        }
    }
}

impl GtkUiWidget for FloTextBoxWidget {
    ///
    /// Retrieves the ID assigned to this widget
    /// 
    fn id(&self) -> WidgetId {
        self.id
    }

    ///
    /// Processes an action for this widget
    ///
    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;

        match action {
            // Entry text can either be set as the value
            State(WidgetState::SetValueText(val)) => {
                self.widget.set_text(val);
            },

            // ... or using the normal text content setting
            Content(WidgetContent::SetText(val)) => {
                self.widget.set_text(val);
            },

            // Generate entry editing events
            RequestEvent(GtkWidgetEventType::EditValue, _event_name) => { 
                self.widget.connect_property_text_notify(|widget| {
                    println!("Edited text: {:?}", widget.get_text());
                });
            }

            // Toggling the button causes a set value event
            RequestEvent(GtkWidgetEventType::SetValue, _event_name) => {

            }

            // Standard behaviour for all other actions
            other_action => { process_basic_widget_action(self, flo_gtk, other_action); }            
        }
    }

    ///
    /// Sets the children of this widget
    /// 
    fn set_children(&mut self, _children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // TextBox widgets cannot have child controls
    }

    ///
    /// Retrieves the underlying widget for this UI widget
    /// 
    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }

}
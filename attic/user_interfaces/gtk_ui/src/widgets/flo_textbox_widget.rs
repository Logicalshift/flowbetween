use super::basic_widget::*;
use super::super::widgets::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::gtk_event_parameter::*;
use super::super::gtk_widget_event_type::*;

use gtk;
use gtk::prelude::*;

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
            RequestEvent(GtkWidgetEventType::EditValue, event_name) => {
                // Every text change generates an 'edit' event
                let id          = self.id;
                let sink        = flo_gtk.get_event_sink();
                let event_name  = event_name.clone();

                self.widget.connect_property_text_notify(move |widget| {
                    let new_text = widget.get_text();
                    let new_text = String::from(new_text);
                    publish_event(&sink, GtkEvent::Event(id, event_name.clone(), GtkEventParameter::NewText(new_text)));
                });
            }

            // Toggling the button causes a set value event
            RequestEvent(GtkWidgetEventType::SetValue, name) => {
                // Editing ends if the user hits enter or focus is lost from the control

                // Focus lost
                let id          = self.id;
                let sink        = flo_gtk.get_event_sink();
                let event_name  = name.clone();

                self.widget.connect_focus_out_event(move |widget, _focus| {
                    let new_text = widget.get_text();
                    let new_text = String::from(new_text);
                    publish_event(&sink, GtkEvent::Event(id, event_name.clone(), GtkEventParameter::NewText(new_text)));

                    Inhibit(false)
                });

                // Hitting the enter key (which Gtk+ considers as activation)
                let id          = self.id;
                let sink        = flo_gtk.get_event_sink();
                let event_name  = name.clone();

                self.widget.connect_activate(move |widget| {
                    let new_text = widget.get_text();
                    let new_text = String::from(new_text);
                    publish_event(&sink, GtkEvent::Event(id, event_name.clone(), GtkEventParameter::NewText(new_text)));
                });
            }

            // Click events are ignored (they focus the control)
            RequestEvent(GtkWidgetEventType::Click, _event_name) => {

            },

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

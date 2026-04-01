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
/// Implements behaviour for the checkbox (checkbutton) widget
///
/// These are checkboxes to Flo but checkbuttons to Gtk, we're using Flo's terminology here.
///
pub struct FloCheckBoxWidget {
    /// The ID of this widget
    id: WidgetId,

    /// The checkbutton widget
    widget: gtk::CheckButton,

    /// The checkbutton again, but cast to a widget
    as_widget: gtk::Widget
}

impl FloCheckBoxWidget {
    ///
    /// Creates a new checkbox widget
    ///
    pub fn new<W: Clone+Cast+IsA<gtk::CheckButton>+IsA<gtk::Widget>>(id: WidgetId, check_button: W) -> FloCheckBoxWidget {
        FloCheckBoxWidget {
            id:             id,
            widget:         check_button.clone().upcast::<gtk::CheckButton>(),
            as_widget:      check_button.clone().upcast::<gtk::Widget>()
        }
    }
}

impl GtkUiWidget for FloCheckBoxWidget {
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
            // Can be checked/unchecked either by setting the 'selected' state or the 'value'
            State(WidgetState::SetSelected(is_selected)) |
            State(WidgetState::SetValueBool(is_selected)) => {
                self.widget.set_active(*is_selected);
            },

            // EditValue events are ignored (there isn't a sensible way to define an 'ongoing' edit of a checkbox)
            RequestEvent(GtkWidgetEventType::EditValue, _) => {
            },

            // Toggling the button causes a set value event
            RequestEvent(GtkWidgetEventType::SetValue, event_name) => {
                let id              = self.id;
                let sink            = flo_gtk.get_event_sink();
                let event_name      = event_name.clone();

                self.widget.connect_toggled(move |widget| {
                    let is_selected = widget.get_active();
                    publish_event(&sink, GtkEvent::Event(id, event_name.clone(), GtkEventParameter::SelectedValue(is_selected)));
                });
            },

            // Click events are ignored (they are handled directly by the control)
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
        // Checkbox widgets in other ports of the UI can't have child controls, although Gtk+ checkboxes can
        // For the moment we won't support them here either
    }

    ///
    /// Retrieves the underlying widget for this UI widget
    ///
    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }

}

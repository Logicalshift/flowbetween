use super::basic_widget::*;
use super::super::widgets::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

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

    /// The checbutton again, but cast to a widget
    as_widget: gtk::Widget,

    /// The widget data
    widget_data: Rc<WidgetData>,
}

impl FloCheckBoxWidget {
    ///
    /// Creates a new checkbox widget
    ///
    pub fn new<W: Clone+Cast+IsA<gtk::CheckButton>+IsA<gtk::Widget>>(id: WidgetId, check_button: W, data: Rc<WidgetData>) -> FloCheckBoxWidget {
        FloCheckBoxWidget {
            id:             id,
            widget:         check_button.clone().upcast::<gtk::CheckButton>(),
            as_widget:      check_button.clone().upcast::<gtk::Widget>(),
            widget_data:    data
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
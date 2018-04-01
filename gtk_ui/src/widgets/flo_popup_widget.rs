use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

// TODO: actually manage popups

///
/// The popup widget is used to manage GTK popup widgets
/// 
pub struct FloPopupWidget {
    /// The ID of the widget
    id: WidgetId,

    /// The popup widget itself
    widget: gtk::Widget
}

impl FloPopupWidget {
    ///
    /// Creates a basic widget
    /// 
    pub fn new<Src: Cast+IsA<gtk::Widget>>(id: WidgetId, widget: Src) -> FloPopupWidget {
        FloPopupWidget {
            id:     id,
            widget: widget.upcast::<gtk::Widget>()
        }
    }
}

impl GtkUiWidget for FloPopupWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        process_basic_widget_action(self, flo_gtk, action);
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // Popup widgets currently do not manage their child widgets, so we just dump these on the floor for now
        // TODO: these eventually go 'inside' the popup
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.widget
    }
}

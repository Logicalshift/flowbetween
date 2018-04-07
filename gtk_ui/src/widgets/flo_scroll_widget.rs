use super::widget::*;
use super::widget_data::*;
use super::flo_fixed_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// The scroll widget manages a layout widget in order to provide a scrollable region
/// 
pub struct FloScrollWidget {
    /// The ID of this widget
    id:         WidgetId,

    /// The layout widget that we'll use to manage the scrolling region
    layout:     gtk::Layout,

    /// The same widget, cast as a widget
    as_widget:  gtk::Widget,

    /// We delegate the actual layout tasks (along with things like setting the image and text) to FloFixedWidget
    fixed_widget: FloFixedWidget
}

impl FloScrollWidget {
    ///
    /// Creates a new scroll widget
    ///
    pub fn new(id: WidgetId, layout: gtk::Layout, widget_data: Rc<WidgetData>) -> FloScrollWidget {
        let as_widget       = layout.clone().upcast::<gtk::Widget>();
        let fixed_widget    = FloFixedWidget::new(id, layout.clone(), widget_data);

        FloScrollWidget {
            id:             id,
            layout:         layout,
            as_widget:      as_widget,
            fixed_widget:   fixed_widget
        }
    }
}

impl GtkUiWidget for FloScrollWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        match action {
            // All other actions act as if the fixed widget performed them
            other_action => { self.fixed_widget.process(flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        self.fixed_widget.set_children(children);
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

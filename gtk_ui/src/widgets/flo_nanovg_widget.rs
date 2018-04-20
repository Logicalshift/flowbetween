use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Uses NanoVG to draw using OpenGL on a widget
/// 
pub struct FloNanoVgWidget {
    /// The ID of this widget
    id: WidgetId,

    /// THis widget represented as a widget
    as_widget: gtk::Widget
}

impl FloNanoVgWidget {
    ///
    /// Creates a new NanoVG widget with a particular GL area as the target
    /// 
    pub fn new<W: Clone+Cast+IsA<gtk::GLArea>>(widget_id: WidgetId, widget: W) -> FloNanoVgWidget {
        let gl_widget = widget.upcast::<gtk::GLArea>();
        let as_widget = gl_widget.clone().upcast::<gtk::Widget>();

        FloNanoVgWidget {
            id: widget_id,
            as_widget: as_widget
        }
    }
}

impl GtkUiWidget for FloNanoVgWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        process_basic_widget_action(self, flo_gtk, action);
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // NanoVG widgets cannot have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}
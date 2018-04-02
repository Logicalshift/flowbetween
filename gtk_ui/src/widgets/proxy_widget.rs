use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;
use std::ops::{Deref, DerefMut};

///
/// Sometimes we want to replace one widget with another. For instance, a widget with a Z-index needs to
/// be placed in an EventBox so that it's associated with a window and can thus be re-ordered.
///
pub struct ProxyWidget<Widget> {
    /// The underlying widget
    underlying_widget: Rc<RefCell<GtkUiWidget>>,

    /// The widget that we're proxying
    proxy_widget: Widget,

    /// The proxy widget represented as a GTK widget 
    as_widget: gtk::Widget
}

impl<Widget: Clone+Cast+IsA<gtk::Widget>> ProxyWidget<Widget> {
    ///
    /// Creates a new proxy widget
    /// 
    pub fn new(underlying_widget: Rc<RefCell<GtkUiWidget>>, proxy_widget: Widget) -> ProxyWidget<Widget> {
        ProxyWidget {
            underlying_widget:  underlying_widget,
            as_widget:          proxy_widget.clone().upcast::<gtk::Widget>(),
            proxy_widget:       proxy_widget
        }
    }
}

impl<Widget> Deref for ProxyWidget<Widget> {
    type Target = Widget;

    fn deref(&self) -> &Widget {
        &self.proxy_widget
    }
}

impl<Widget> DerefMut for ProxyWidget<Widget> {
    fn deref_mut(&mut self) -> &mut Widget {
        &mut self.proxy_widget
    }
}

impl<Widget> GtkUiWidget for ProxyWidget<Widget> {
    fn id(&self) -> WidgetId {
        self.underlying_widget.borrow().id()
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;

        // Some actions should always be processed against the proxy widget
        match action {
            // The proxy widget should become the root, not its content
            &SetRoot(_)         => { process_basic_widget_action(self, flo_gtk, action); },

            // Some appearance settings (like background colour) can only be set on things like EventBoxes, so the proxy processes them
            &Appearance(_)      => { process_basic_widget_action(self, flo_gtk, action); },

            // Everything else is processed like the proxy doesn't exist
            _                   => { self.underlying_widget.borrow_mut().process(flo_gtk, action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        self.underlying_widget.borrow_mut().set_children(children)
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

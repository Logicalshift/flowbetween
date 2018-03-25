use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use gtk;

use std::rc::*;
use std::cell::*;

///
/// Trait implemented by objects that can act as widgets
/// 
pub trait GtkUiWidget {
    ///
    /// Retrieves the ID assigned to this widget
    /// 
    fn id(&self) -> WidgetId;

    ///
    /// Processes an action for this widget
    ///
    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction);

    ///
    /// Sets the children of this widget
    /// 
    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>);

    ///
    /// Retrieves the underlying widget for this UI widget
    /// 
    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget;
}

impl GtkUiWidget for Box<GtkUiWidget> {
    fn id(&self) -> WidgetId                                                { (**self).id() }
    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction)   { (**self).process(flo_gtk, action) }
    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>)     { (**self).set_children(children) }
    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget                      { (**self).get_underlying() }
}

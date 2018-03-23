use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use gtk;

///
/// Trait implemented by objects that can act as widgets
/// 
pub trait GtkUiWidget {
    ///
    /// Processes an action for this widget
    ///
    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction);

    ///
    /// Adds a child widget to this widget
    /// 
    fn add_child(&mut self, new_child: &gtk::Widget);
}
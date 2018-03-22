use super::super::gtk_action::*;
use super::super::gtk_thread::*;

///
/// Trait implemented by objects that can act as widgets
/// 
pub trait GtkUiWidget {
    ///
    /// Processes an action for this widget
    ///
    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction);
}
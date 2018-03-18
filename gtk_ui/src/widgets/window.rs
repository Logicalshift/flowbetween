use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use gtk;

///
/// Trait implemented by objects that can act as windows
/// 
pub trait GtkUiWindow {
    ///
    /// Processes an action for this window
    /// 
    fn process(&mut self, flo_gtk: &FloGtk, action: GtkWindowAction);
}

impl GtkUiWindow for gtk::Window {
    fn process(&mut self, flo_gtk: &FloGtk, action: GtkWindowAction) {
        unimplemented!()
    }
}
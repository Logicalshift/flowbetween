use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use gtk;
use gtk::prelude::*;

///
/// Trait implemented by objects that can act as windows
/// 
pub trait GtkUiWindow {
    ///
    /// Processes an action for this window
    /// 
    fn process(&mut self, flo_gtk: &FloGtk, action: &GtkWindowAction);
}

impl GtkUiWindow for gtk::Window {
    fn process(&mut self, _flo_gtk: &FloGtk, action: &GtkWindowAction) {
        match action {
            &GtkWindowAction::New(ref _window_type)         => { },
            &GtkWindowAction::SetTitle(ref title)           => { self.set_title(&*title); },
            &GtkWindowAction::SetDefaultSize(width, height) => { self.set_default_size(width, height); },
            &GtkWindowAction::SetPosition(pos)              => { self.set_position(pos.clone()); },
            &GtkWindowAction::ShowAll                       => { self.show_all(); },
            &GtkWindowAction::Hide                          => { self.hide(); },
            &GtkWindowAction::Close                         => { self.hide(); }
        }
    }
}

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
    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWindowAction);

    ///
    /// Sets a widhget as the root of this window
    ///
    fn set_root(&mut self, flo_gtk: &mut FloGtk, widget: &gtk::Widget);
}

impl GtkUiWindow for gtk::Window {
    fn process(&mut self, _flo_gtk: &mut FloGtk, action: &GtkWindowAction) {
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

    fn set_root(&mut self, flo_gtk: &mut FloGtk, widget: &gtk::Widget) {
        // Replace any existing child of this window with the specified widget
        self.get_child().map(|child| self.remove(&child));
        self.add(widget);

        widget.show_all();
    }
}

use super::flo_gtk::*;
use super::super::gtk_action::*;

use gtk;

///
/// Executes a Gtk action
/// 
pub fn run_action(flo_gtk: &mut FloGtk, action: &GtkAction) {
    match action {
        &GtkAction::Stop                                    => gtk::main_quit(),
        &GtkAction::Window(window_id, ref window_action)    => run_window_action(flo_gtk, window_id, window_action)
    }
}

///
/// Executes a Gtk window action
/// 
fn run_window_action(flo_gtk: &mut FloGtk, window_id: WindowId, action: &GtkWindowAction) {
    match action {
        &GtkWindowAction::New(ref window_type) => {
            // For new window actions, we need to create the window before we proceed
            let new_window = gtk::Window::new(window_type.clone());
            flo_gtk.register_window(window_id, new_window);

            flo_gtk.get_window(window_id).map(|mut window| window.process(flo_gtk, &GtkWindowAction::New(window_type.clone())));
        },

        &GtkWindowAction::Close => {
            // Closing the window removes it entirely from the windows we know about
            flo_gtk.get_window(window_id).map(|mut window| window.process(flo_gtk, &GtkWindowAction::Close));
            flo_gtk.remove_window(window_id);
        },

        other => {
            // For all other actions, we just pass on to the window with this ID
            flo_gtk.get_window(window_id).map(|mut window| window.process(flo_gtk, other));
        }
    }
}
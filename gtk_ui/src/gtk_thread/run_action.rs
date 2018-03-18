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
        &GtkWindowAction::New(ref window_type)          => unimplemented!(),
        &GtkWindowAction::SetTitle(ref title)           => unimplemented!(),
        &GtkWindowAction::SetDefaultSize(width, height) => unimplemented!(),
        &GtkWindowAction::SetPosition(pos)              => unimplemented!(),
        &GtkWindowAction::ShowAll                       => unimplemented!(),
        &GtkWindowAction::Hide                          => unimplemented!(),
        &GtkWindowAction::Close                         => unimplemented!()
    }
}
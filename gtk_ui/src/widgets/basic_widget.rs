use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use gtk::*;

///
/// Performs the basic processing associated with a widget action (using a generic Gtk widget as the target)
/// 
pub fn process_basic_widget_action(widget: &mut Widget, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
    use self::GtkWidgetAction::*;

    match action {
        &New(_widget_type)          => (),
        &SetRoot(window_id)         => { flo_gtk.get_window(window_id).map(|window| window.borrow_mut().set_root(flo_gtk, widget)); },
        &Layout(ref layout)         => unimplemented!(),
        &Content(ref content)       => unimplemented!(),
        &Appearance(ref appearance) => unimplemented!(),
        &State(ref state)           => unimplemented!(),
        &Font(ref font)             => unimplemented!(),
        &Delete                     => unimplemented!()
    }
}

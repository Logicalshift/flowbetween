use super::flo_gtk::*;
use super::super::gtk_action::*;

use gtk;
use glib;
use gtk::prelude::*;

///
/// Executes a Gtk action
/// 
pub fn run_action(flo_gtk: &mut FloGtk, action: &GtkAction) {
    match action {
        &GtkAction::Stop                                    => gtk::main_quit(),
        &GtkAction::Window(window_id, ref window_action)    => run_window_action(flo_gtk, window_id, window_action),
        &GtkAction::Widget(widget_id, ref widget_action)    => run_widget_action(flo_gtk, widget_id, widget_action)
    }
}

///
/// Executes a Gtk window action
/// 
fn run_window_action(flo_gtk: &mut FloGtk, window_id: WindowId, actions: &Vec<GtkWindowAction>) {
    // Fetch the window with this ID
    let mut window = flo_gtk.get_window(window_id);

    // Send the actions to it
    for action in actions.iter() {
        match action {
            &GtkWindowAction::New(ref window_type) => {
                // For new window actions, we need to create the window before we can send actions to it
                let new_window = gtk::Window::new(window_type.clone());

                // Add our style context
                new_window.get_style_context()
                    .unwrap()
                    .add_provider(flo_gtk.style_provider(), gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

                // Store the window ID
                new_window.set_property("flo_window_id", &glib::AnyValue::new(window_id)).unwrap();
                
                // Register the window
                flo_gtk.register_window(window_id, new_window);

                // Fetch the reference to the new window and make it the reference for the rest of the commands
                window = flo_gtk.get_window(window_id);

                // Send the 'new' request to the newly created window
                window.as_mut().map(|window| window.borrow_mut().process(flo_gtk, &GtkWindowAction::New(window_type.clone())));
            },

            &GtkWindowAction::Close => {
                // Closing the window removes it entirely from the windows we know about
                window.as_mut().map(|window| window.borrow_mut().process(flo_gtk, &GtkWindowAction::Close));
                flo_gtk.remove_window(window_id);
            },

            other => {
                // For all other actions, we just pass on to the window with this ID
                window.as_mut().map(|window| window.borrow_mut().process(flo_gtk, other));
            }
        }
    }
}

///
/// Executes a Gtk widget action
/// 
fn run_widget_action(flo_gtk: &mut FloGtk, widget_id: WidgetId, actions: &Vec<GtkWidgetAction>) {
    unimplemented!()
}
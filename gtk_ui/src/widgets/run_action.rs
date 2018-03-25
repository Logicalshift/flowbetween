use super::factory::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use gtk;
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
                let mut new_window = gtk::Window::new(window_type.clone());

                // Add our style context
                new_window.get_style_context()
                    .unwrap()
                    .add_provider(flo_gtk.style_provider(), gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

                // New windows with no content get a generic message initially
                new_window.add(&gtk::Label::new("Flo: This space left blank"));
                
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
                window.as_ref().map(|window| window.borrow_mut().process(flo_gtk, other));
            }
        }
    }
}

///
/// Executes a Gtk widget action
/// 
fn run_widget_action(flo_gtk: &mut FloGtk, widget_id: WidgetId, actions: &Vec<GtkWidgetAction>) {
    // Fetch the widget for which we will be dispatching actions
    let widget_data = flo_gtk.widget_data();
    let mut widget  = widget_data.get_widget(widget_id);

    // Send the actions to the widget
    for action in actions.iter() {
        match action {
            &GtkWidgetAction::New(widget_type)  => {
                // Call the factory method to create a new widget
                let new_widget = create_widget(widget_id, widget_type);

                // Register with the widget data
                widget_data.register_widget(widget_id, new_widget);
                
                // Update the widget that actions are sent to
                widget = widget_data.get_widget(widget_id);

                // Dispatch the 'new' action to the newly created widget
                widget.as_ref().map(|widget| widget.borrow_mut().process(flo_gtk, &GtkWidgetAction::New(widget_type)));
            },

            &GtkWidgetAction::Delete => {
                // Send the delete request to this widget so it can do whatever it needs to do
                widget.as_ref().map(|widget| widget.borrow_mut().process(flo_gtk, &GtkWidgetAction::Delete));

                // Remove the data for this widget
                widget_data.remove_widget(widget_id);
                widget = None;
            },

            other => {
                // Other actions can just be sent straight to the widget involved
                widget.as_ref().map(|widget| widget.borrow_mut().process(flo_gtk, other));
            }
        }
    }
}
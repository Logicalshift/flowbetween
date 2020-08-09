use super::image::*;
use super::factory::*;
use super::super::gtk_event::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;
use super::super::widgets::custom_style::*;
use super::super::widgets::proxy_widget::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;

lazy_static! {
    /// Hard coded icon we use (should make this selectable through the UI interface)
    static ref ICON_IMAGE: StaticImageData = StaticImageData::new(include_bytes!["../../../../png/Flo-Orb-small.png"]);
}

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
/// Wires up the events for a window
///
fn wire_up_window(new_window: &gtk::Window, window_id: WindowId, flo_gtk: &mut FloGtk) {
    // Retrieve a sink where we can send our events to
    let event_sink = flo_gtk.get_event_sink();

    // Send the close event when the window is closed
    new_window.connect_hide(move |_window| { publish_event(&event_sink, GtkEvent::CloseWindow(window_id)); });
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

                // Set the icon
                let icon = pixbuf_from_png(&*ICON_IMAGE);
                new_window.set_icon(Some(&icon));

                // New windows with no content get a generic message initially
                new_window.add(&gtk::Label::new(Some("Flo: This space left blank")));

                // Wire up events
                wire_up_window(&new_window, window_id, flo_gtk);

                // Add our style context
                new_window.get_style_context()
                    .add_provider(flo_gtk.style_provider(), gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

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
                // If there's an existing widget with this ID, delete it
                if widget.is_some() {
                    // Cause the current widget to be deleted
                    widget.as_ref().map(|widget| widget.borrow_mut().process(flo_gtk, &GtkWidgetAction::Delete));
                    widget_data.remove_widget(widget_id);
                }

                // Call the factory method to create a new widget
                let new_widget = create_widget(widget_id, widget_type, Rc::clone(&widget_data));

                // Add our standard style provider
                new_widget.get_underlying().get_style_context()
                    .add_provider(flo_gtk.style_provider(), gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

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

            &GtkWidgetAction::IntoEventBox => {
                // Boxing a widget creates a new event box with the old widget inside
                if let Some(widget_ref) = widget.as_ref() {
                    // Create a clone of the widget object
                    let gtk_widget  = widget_ref.borrow().get_underlying().clone();

                    // If it's in a container, fetch that so we can shuffle things around
                    let container   = gtk_widget.get_parent().and_then(|parent| parent.dynamic_cast::<gtk::Container>().ok());

                    // No window. Wrap in an event box, which always has its own window
                    let event_box = gtk::EventBox::new();

                    container.as_ref().map(|container| container.remove(&gtk_widget));
                    event_box.add(&gtk_widget);

                    container.as_ref().map(|container| container.add(&event_box));

                    // Ensure it has the same visibility as the parent widget
                    event_box.set_visible(gtk_widget.get_visible());

                    // Substitute a proxy widget
                    let proxy_event_box = ProxyWidget::new(Rc::clone(widget_ref), event_box);
                    widget_data.replace_widget(widget_id, proxy_event_box);
                }

                // Use the substitute widget for future updates
                widget = widget_data.get_widget(widget_id);
            },

            other => {
                // Other actions can just be sent straight to the widget involved
                widget.as_ref().map(|widget| widget.borrow_mut().process(flo_gtk, other));
            }
        }
    }

    // Perform post-processing steps
    widget.as_ref().map(|widget| widget_data.update_custom_style(&*widget.borrow()));
}

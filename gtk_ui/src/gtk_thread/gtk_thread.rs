use super::flo_gtk::*;

use gtk;

use std::thread;
use std::thread::JoinHandle;

///
/// Represents a running Gtk thread, providing an interface for other threads to use
/// 
pub struct GtkThread {
    /// Used to send messages and actions to the Gtk thread
    message_target: GtkMessageTarget
}

impl GtkThread {
    ///
    /// Creates a new Gtk thread
    /// 
    pub fn new() -> GtkThread {
        GtkThread {
            message_target: GtkMessageTarget::new()
        }
    }

    ///
    /// Starts running Gtk in a thread
    /// 
    pub fn run_thread(&self) -> JoinHandle<()> {
        // Clone the message target so we can use it as the source for the new thread
        let thread_target = self.message_target.clone();

        // Start the Gtk thread
        let thread = thread::spawn(move || {
            // Create the Gtk data structure
            let flo_gtk = FloGtk::new();

            // Send messages to it
            flo_gtk.receive_messages(&thread_target);

            // Start gtk running
            gtk::main();
        });

        thread
    }
}
use super::flo_gtk::*;

use gtk;

use std::thread;
use std::thread::JoinHandle;

///
/// Represents a running Gtk thread, providing an interface for other threads to use
/// 
pub struct GtkThread {
    /// Used to send messages and actions to the Gtk thread
    message_target: GtkMessageTarget,

    /// None, or the running thread
    running_thread: Option<JoinHandle<()>>
}

impl GtkThread {
    ///
    /// Creates a new Gtk thread
    /// 
    pub fn new() -> GtkThread {
        // Create a new thread
        let mut thread = GtkThread {
            message_target: GtkMessageTarget::new(),
            running_thread: None
        };

        // Start it running
        thread.running_thread = Some(thread.run_thread());

        // This thread is the result
        thread
    }

    ///
    /// Starts running Gtk in a thread
    /// 
    fn run_thread(&self) -> JoinHandle<()> {
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

impl Drop for GtkThread {
    fn drop(&mut self) {
        // When a GtkThread is dropped, tell GTK to shut down
        self.message_target.async(|_gtk| gtk::main_quit());

        // Wait for the thread to finish before the object is truely dropped
        self.running_thread.take().map(|running_thread| running_thread.join());
    }
}
use super::flo_gtk::*;
use super::event_sink::*;
use super::super::gtk_action::*;
use super::super::widgets::*;

use gtk;

use std::thread;
use std::thread::JoinHandle;

///
/// Represents a running Gtk thread, providing an interface for other threads to use
/// 
pub struct GtkThread {
    /// A clone of the event sink that the Gtk thread will send its events to
    event_sink: GtkEventSink,

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
        // Create the event sink
        let event_sink = GtkEventSink::new();

        // Create a new thread
        let mut thread = GtkThread {
            event_sink:     event_sink,
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
        let thread_target   = self.message_target.clone();
        let event_sink      = self.event_sink.clone();

        // Start the Gtk thread
        let thread = thread::spawn(move || {
            // Initialise gtk and panic if we get a failure
            let init_result = gtk::init();
            if init_result.is_err() {
                panic!("Failed to start GTK: {:?}", init_result);
            }

            // Create the Gtk data structure
            let flo_gtk = FloGtk::new(event_sink);

            // Send messages to it
            flo_gtk.receive_messages(&thread_target);

            // Start gtk running
            gtk::main();
        });

        thread
    }

    ///
    /// Performs a set of actions on the Gtk thread
    /// 
    pub fn perform_actions(&mut self, actions: Vec<GtkAction>) {
        self.message_target.async(|flo_gtk| {
            for action in actions {
                run_action(flo_gtk, &action)
            }
        });
    }

    ///
    /// Retrieves a stream of the events originating from the GTK thread
    /// 
    pub fn get_event_stream(&self) -> GtkEventStream {
        self.event_sink.get_stream()
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
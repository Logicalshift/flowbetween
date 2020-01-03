use super::flo_gtk::*;
use super::event_sink::*;
use super::super::gtk_action::*;
use super::super::gtk_event::*;
use super::super::widgets::*;

use ::desync::*;
use flo_stream::*;

use gl;
use gtk;
use epoxy;
use futures::*;
use futures::stream::{BoxStream};
use shared_library::dynamic_library::DynamicLibrary;

use std::sync::*;
use std::thread;
use std::thread::JoinHandle;
use std::ptr;

///
/// Represents a running Gtk thread, providing an interface for other threads to use
///
pub struct GtkThread {
    /// A clone of the event sink that the Gtk thread will send its events to
    event_sink: Arc<Desync<Publisher<GtkEvent>>>,

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
        let event_sink = Publisher::new(100);

        // Create a new thread
        let mut thread = GtkThread {
            event_sink:     Arc::new(Desync::new(event_sink)),
            message_target: GtkMessageTarget::new(),
            running_thread: None
        };

        // Start it running
        thread.running_thread = Some(thread.run_thread());

        // This thread is the result
        thread
    }

    ///
    /// Sets up OpenGL on a GTK thread
    ///
    /// See https://github.com/gtk-rs/examples/pull/44/files
    ///
    fn initialize_opengl() {
        // Tell epoxy how to load symbols from this process
        epoxy::load_with(|symbol_name| {
            unsafe {
                match DynamicLibrary::open(None).unwrap().symbol(symbol_name) {
                    Ok(v)   => v,
                    Err(_)  => ptr::null(),
                }
            }
        });

        // Load OpenGL via epoxy
        gl::load_with(epoxy::get_proc_addr);
    }

    ///
    /// Starts running Gtk in a thread
    ///
    fn run_thread(&self) -> JoinHandle<()> {
        // Clone the message target so we can use it as the source for the new thread
        let thread_target   = self.message_target.clone();
        let event_sink      = Arc::new(Desync::new(self.event_sink.sync(|sink| sink.republish_weak())));

        // Start the Gtk thread
        let thread = thread::spawn(move || {
            // Initialise gtk and panic if we get a failure
            let init_result = gtk::init();
            if init_result.is_err() {
                panic!("Failed to start GTK: {:?}", init_result);
            }

            // Prepare OpenGL as well
            Self::initialize_opengl();

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
    pub fn perform_actions(&self, actions: Vec<GtkAction>) {
        if actions.len() > 0 {
            self.message_target.desync(|flo_gtk| {
                // Run all of the actions
                for action in actions {
                    run_action(flo_gtk, &action)
                }

                // Generate a tick event when they're complete
                publish_event(&flo_gtk.get_event_sink(), GtkEvent::Tick);
            });
        }
    }

    ///
    /// Retrieves a stream of the events originating from the GTK thread
    ///
    pub fn get_event_stream(&self) -> BoxStream<'static, GtkEvent> {
        self.event_sink.sync(|sink| sink.subscribe()).boxed()
    }
}

impl Drop for GtkThread {
    fn drop(&mut self) {
        // When a GtkThread is dropped, tell GTK to shut down
        self.message_target.desync(|_gtk| gtk::main_quit());

        // Wait for the thread to finish before the object is truely dropped
        self.running_thread.take().map(|running_thread| running_thread.join());
    }
}

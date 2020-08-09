use super::message::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;
use super::super::gtk_event::*;
use super::super::widgets::*;

use flo_stream::*;

use gtk;
use gtk::prelude::*;
use glib;
use futures::prelude::*;
use futures::stream::{BoxStream};

use std::collections::{HashMap, VecDeque};
use std::cell::RefCell;
use std::sync::*;
use std::rc::Rc;
use std::thread;

// Contains the FloGtk instance running on the current thread
thread_local!(static GTK_INSTANCES: RefCell<Vec<FloGtk>> = RefCell::new(vec![]));

/// Queue of messages waiting to be sent to the GTK thread
#[derive(Clone)]
struct MessageQueue(Arc<Mutex<VecDeque<Box<dyn FloGtkMessage>>>>);

///
/// Represents a target for GTK-related messages
///
#[derive(Clone)]
pub struct GtkMessageTarget {
    /// The queue where messages for this target will be sent
    queue: MessageQueue,

    // Context where messages will be sent
    target_context: Option<glib::MainContext>
}

///
/// Data storage structures associated with a FlowBetween gtk session
///
/// This represents the main gtk thread (which is not usually the general main thread)
///
pub struct FloGtk {
    /// Messages pending for the GTK thread
    pending_messages: MessageQueue,

    /// Hashmap for the windows that are being managed by this object
    windows: HashMap<WindowId, Rc<RefCell<dyn GtkUiWindow>>>,

    /// Data associated with Flo widgets
    widget_data: Rc<WidgetData>,

    /// The event sink for this object
    event_sink: GtkEventSink,

    /// The style provider for the widgets and windows created for this Gtk instance
    style_provider: gtk::CssProvider
}

impl MessageQueue {
    pub fn new() -> MessageQueue {
        MessageQueue(Arc::new(Mutex::new(VecDeque::new())))
    }
}

impl GtkMessageTarget {
    ///
    /// Creates a new GTK message target
    ///
    pub fn new() -> GtkMessageTarget {
        GtkMessageTarget {
            queue:          MessageQueue::new(),
            target_context: Some(glib::MainContext::default())
        }
    }

    ///
    /// Performs an action asynchronously on this message target
    ///
    pub fn desync<MsgFn: 'static+Send+FnOnce(&mut FloGtk) -> ()>(&self, action: MsgFn) {
        // Lock the queue in order to start sending the message
        let mut queue = self.queue.0.lock().unwrap();

        // If a message is already on the queue, then the thread is already set to wake and we don't need to wake it again
        let messages_pending = queue.len() > 0;

        // Append the action to the queue
        queue.push_back(Box::new(FnOnceMessage::new(action)));

        // Wake the thread and tell it to process messages if needed
        if !messages_pending {
            // The idle_add_full function would be more elegant here but it's not currently available in glib
            // (idle_add uses too low a priority for us)
            let source = glib::idle_source_new(Some("flo"), glib::PRIORITY_HIGH, process_pending_messages);
            source.attach(self.target_context.as_ref());
        }
    }

    ///
    /// Performs an action synchronously on this message target
    ///
    pub fn sync<TReturn: 'static+Send, MsgFn: 'static+Send+FnOnce(&mut FloGtk) -> TReturn>(&self, action: MsgFn) -> TReturn {
        // Thread to be woken once the event is available
        let wake_thread = thread::current();

        // The result will be placed here
        let our_result = Arc::new(Mutex::new(None));

        // We pass a copy of this result to the target thread
        let gtk_result = our_result.clone();

        // Dispatch the task to the gtk thread
        self.desync(move |gtk| {
            // Fetch the result from the function
            let result = action(gtk);

            // Store as the thread result
            *gtk_result.lock().unwrap() = Some(result);

            // Unpark the wake thread when done
            wake_thread.unpark();
        });

        // Park the thread until the result is available
        // (If unpark is called before park, the park call should return immediately)
        while { our_result.lock().unwrap().is_none() } {
            thread::park();
        }

        // Return the result that was generated on the gtk thread
        let result = our_result.lock().unwrap().take();
        result.unwrap()
    }
}

///
/// Callback function that tells all of the FloGtk objects on the current thread to process their pending messages
///
fn process_pending_messages() -> glib::Continue {
    GTK_INSTANCES.with(|gtk_instances| {
        // Tell each instance on this thread to process its pending messages immediately
        for instance in gtk_instances.borrow_mut().iter_mut() {
            instance.process_messages();
        }
    });

    glib::Continue(false)
}

impl FloGtk {
    ///
    /// Creates a new FloGtk instance
    ///
    pub fn new(event_sink: GtkEventSink) -> FloGtk {
        let mut style_provider = gtk::CssProvider::new();
        style_provider.load_from_data(include_bytes!["../../style/flo.css"]).unwrap();

        FloGtk {
            pending_messages:   MessageQueue::new(),
            windows:            HashMap::new(),
            widget_data:        Rc::new(WidgetData::new()),
            event_sink:         event_sink,
            style_provider:     style_provider
        }
    }

    ///
    /// Sets this FloGtk object as the GTK instance for the current thread and begins receiving messages for it
    /// from the specified message target
    ///
    pub fn receive_messages(mut self, source: &GtkMessageTarget) {
        // Receive messages from the source
        self.pending_messages = source.queue.clone();

        // Add this to the receivers for the current thread
        GTK_INSTANCES.with(move |instances| instances.borrow_mut().push(self));

        // Ensure that we're ready to go by flushing all pending messages for this thread immediately
        // If there were any messages pending before we added to the list of instances, this thread will never be triggered
        process_pending_messages();
    }

    ///
    /// Retrieves the style provider for this object
    ///
    pub fn style_provider<'a>(&'a mut self) -> &'a mut gtk::CssProvider {
        &mut self.style_provider
    }

    ///
    /// Associates a window with an ID
    ///
    pub fn register_window<TWindow: 'static+GtkUiWindow>(&mut self, window_id: WindowId, window: TWindow) {
        self.windows.insert(window_id, Rc::new(RefCell::new(window)));
    }

    ///
    /// Attempts to retrieve the window with the specified ID
    ///
    pub fn get_window(&self, window_id: WindowId) -> Option<Rc<RefCell<dyn GtkUiWindow>>> {
        self.windows.get(&window_id).cloned()
    }

    ///
    /// Removes the window that has the specified ID
    ///
    pub fn remove_window(&mut self, window_id: WindowId) {
        self.windows.remove(&window_id);
    }

    ///
    /// Retrieves the widget data structure for this object
    ///
    pub fn widget_data(&self) -> Rc<WidgetData> {
        Rc::clone(&self.widget_data)
    }

    ///
    /// Retrieves a stream that will return all future events generated for this object
    ///
    pub fn get_event_stream(&mut self) -> BoxStream<'static, GtkEvent> {
        // Generate a stream from our event sink
        self.event_sink.sync(|sink| sink.subscribe()).boxed()
    }

    ///
    /// Retrieves a sink that can be used to send events to any attached streams
    ///
    pub fn get_event_sink(&mut self) -> GtkEventSink {
        Arc::clone(&self.event_sink)
    }

    ///
    /// Processes any messages pending for this instance
    ///
    fn process_messages(&mut self) {
        // Fetch the current set of pending messages
        let pending_messages = {
            let mut result  = vec![];
            let mut pending = self.pending_messages.0.lock().unwrap();

            while let Some(action) = pending.pop_front() {
                result.push(action);
            }

            result
        };

        // Perform all of the actions
        for mut action in pending_messages {
            action.process(self)
        }
    }
}

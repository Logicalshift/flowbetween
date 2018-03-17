use super::message::*;

use gtk;
use glib;

use std::collections::VecDeque;
use std::cell::RefCell;
use std::sync::*;

/// Contains the FloGtk instance running on the current thread
thread_local!(static GTK_INSTANCES: RefCell<Vec<FloGtk>> = RefCell::new(vec![]));

/// Queue of messages waiting to be sent to the GTK thread
#[derive(Clone)]
struct MessageQueue(Arc<Mutex<VecDeque<Box<FloGtkMessage>>>>);

///
/// Represents a target for GTK-related messages
/// 
#[derive(Clone)]
pub struct GtkMessageTarget {
    /// The queue where messages for this target will be sent
    queue: MessageQueue
}

///
/// Data storage structures associated with a FlowBetween gtk session
/// 
/// This represents the main gtk thread (which is not usually the general main thread)
/// 
pub struct FloGtk {
    /// Messages pending for the GTK thread
    pending_messages: MessageQueue
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
            queue: MessageQueue::new()
        }
    }

    ///
    /// Performs an action asynchronously on this message target
    /// 
    pub fn async<MsgFn: 'static+Send+FnOnce(&FloGtk) -> ()>(&mut self, action: MsgFn) 
    where for<'r> MsgFn: FnOnce(&'r mut FloGtk) -> () {
        // Lock the queue in order to start sending the message
        let mut queue = self.queue.0.lock().unwrap();

        // If a message is already on the queue, then the thread is already set to wake and we don't need to wake it again
        let messages_pending = queue.len() > 0;

        // Append the action to the queue
        queue.push_back(Box::new(FnOnceMessage::new(action)));

        // Wake the thread and tell it to process messages if needed
        if !messages_pending {
            glib::idle_add(process_pending_messages);
        }
    }
}

///
/// Callback function that tells all of the FloGtk objects on the current thread to process their pending messages
/// 
fn process_pending_messages() -> gtk::Continue {
    GTK_INSTANCES.with(|gtk_instances| {
        // Tell each instance on this thread to process its pending messages immediately
        for instance in gtk_instances.borrow_mut().iter_mut() {
            instance.process_messages();
        }
    });

    gtk::Continue(false)
}

impl FloGtk {
    ///
    /// Creates a new FloGtk instance
    /// 
    pub fn new() -> FloGtk {
        FloGtk { 
            pending_messages: MessageQueue::new()
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

use super::traits::*;
use super::releasable::*;
use super::binding_context::*;

use futures::*;
use desync::*;

use std::sync::*;

///
/// Uses a stream to update a binding
/// 
pub fn bind_stream<S, Value, UpdateFn>(stream: S, initial_value: Value, update: UpdateFn) -> StreamBinding<Value>
where   S:          'static+Send+Stream,
        Value:      'static+Send+Clone+PartialEq,
        UpdateFn:   'static+Send+FnMut(Value, S::Item) -> Value,
        S::Item:    Send,
        S::Error:   Send {
    // Create the content of the binding
    let core        = StreamBindingCore {
        value:          initial_value,
        notifications:  vec![]
    };

    let core        = Arc::new(Desync::new(core));
    let mut update  = update;

    // Send in the stream
    pipe_in(Arc::clone(&core), stream, 
        move |core, next_item| {
            if let Ok(next_item) = next_item {
                // Update the value
                let new_value = update(core.value.clone(), next_item);

                if new_value != core.value {
                    // Update the value in the core
                    core.value = new_value;

                    // Notify anything that's listening
                    core.notifications.retain(|notify| notify.is_in_use());
                    core.notifications.iter().for_each(|notify| { notify.mark_as_changed(); });
                }
            } else {
                // TODO: stream errors are currently ignored (not clear if we should handle them or not)
            }
        });
    
    StreamBinding {
        core: core
    }
}

///
/// Binding that represents the result of binding a stream to a value
/// 
#[derive(Clone)]
pub struct StreamBinding<Value: Send> {
    /// The current value of this binding
    core: Arc<Desync<StreamBindingCore<Value>>>
}

///
/// The data stored with a stream binding
/// 
struct StreamBindingCore<Value: Send> {
    /// The current value of this binidng
    value: Value,

    /// The items that should be notified when this binding changes
    notifications: Vec<ReleasableNotifiable>
}

impl<Value: 'static+Send+Clone> Bound<Value> for StreamBinding<Value> {
    ///
    /// Retrieves the value stored by this binding
    ///
    fn get(&self) -> Value {
        BindingContext::add_dependency(self.clone());

        self.core.sync(|core| core.value.clone())
    }
}

impl<Value: 'static+Send> Changeable for StreamBinding<Value> {
    ///
    /// Supplies a function to be notified when this item is changed
    ///
    fn when_changed(&self, what: Arc<dyn Notifiable>) -> Box<dyn Releasable> {
        // Create the notification object
        let releasable = ReleasableNotifiable::new(what);
        let notifiable = releasable.clone_as_owned();

        // Send to the core
        self.core.async(move |core| {
            core.notifications.push(notifiable);
        });

        // Return the releasable object
        Box::new(releasable)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::notify_fn::*;

    use futures::stream;
    use futures::executor;
    use futures::sync::mpsc;

    use std::thread;
    use std::time::Duration;

    #[test]
    pub fn stream_in_all_values() {
        // Stream with the values '1,2,3'
        let stream  = vec![1, 2, 3];
        let stream  = stream::iter_ok::<_, ()>(stream.into_iter());

        // Send the stream to a new binding
        let binding = bind_stream(stream, 0, |_old_value, new_value| new_value);

        thread::sleep(Duration::from_millis(10));

        // Binding should have the value of the last value in the stream
        assert!(binding.get() == 3);
    }

    #[test]
    pub fn stream_processes_updates() {
        // Stream with the values '1,2,3'
        let stream  = vec![1, 2, 3];
        let stream  = stream::iter_ok::<_, ()>(stream.into_iter());

        // Send the stream to a new binding (with some processing)
        let binding = bind_stream(stream, 0, |_old_value, new_value| new_value + 42);

        thread::sleep(Duration::from_millis(10));

        // Binding should have the value of the last value in the stream
        assert!(binding.get() == 45);
    }

    #[test]
    pub fn notifies_on_change() {
        // Create somewhere to send our notifications
        let (sender, receiver) = mpsc::channel(0);

        // Send the receiver stream to a new binding
        let binding = bind_stream(receiver, 0, |_old_value, new_value| new_value);

        // Create the notification
        let notified        = Arc::new(Mutex::new(false));
        let also_notified   = Arc::clone(&notified);

        binding.when_changed(notify(move || *also_notified.lock().unwrap() = true)).keep_alive();

        // Should be initially un-notified
        thread::sleep(Duration::from_millis(5));
        assert!(*notified.lock().unwrap() == false);

        // Send a value to the sender
        let mut sender = executor::spawn(sender);
        sender.wait_send(42).unwrap();

        // Should get notified
        thread::sleep(Duration::from_millis(5));
        assert!(*notified.lock().unwrap() == true);
        assert!(binding.get() == 42);
    }

    #[test]
    pub fn no_notification_on_no_change() {
        // Create somewhere to send our notifications
        let (sender, receiver) = mpsc::channel(0);

        // Send the receiver stream to a new binding
        let binding = bind_stream(receiver, 0, |_old_value, new_value| new_value);

        // Create the notification
        let notified        = Arc::new(Mutex::new(false));
        let also_notified   = Arc::clone(&notified);

        binding.when_changed(notify(move || *also_notified.lock().unwrap() = true)).keep_alive();

        // Should be initially un-notified
        thread::sleep(Duration::from_millis(5));
        assert!(*notified.lock().unwrap() == false);

        // Send a value to the sender. This leaves the final value the same, so no notification should be generated.
        let mut sender = executor::spawn(sender);
        sender.wait_send(0).unwrap();

        // Should get notified
        thread::sleep(Duration::from_millis(5));
        assert!(*notified.lock().unwrap() == false);
        assert!(binding.get() == 0);
    }
}

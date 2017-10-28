use super::traits::*;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::marker::PhantomData;
use futures::{Stream, Poll};
use futures::task;
use futures::task::Task;
use futures::Async::*;

///
/// Turns a binding into a stream. These are streams that return values whenever their binding is
/// updated (ie, producing a stream of the values of a particular bound value).
/// 
/// Bindings are lazy, so this won't catch every single update that happens. Rather, the streaming
/// behaviour is that the stream becomes ready to read at the point the binding changes, and the
/// value read is whatever value is most recent at that point (this is how you avoid updating a
/// user interface with intermediate values, for instance)
///
pub struct BindingStream<Value, Binding> where Binding: Bound<Value> {
    phantom_value: PhantomData<Value>,

    /// The binding that this is streaming values to
    binding: Binding,

    /// Set to true when this is ready
    ready: Arc<ReadyNotification>,

    // How long the notification lasts
    ready_lifetime: Box<Releasable>
}

///
/// Used as the notification target
///
struct ReadyNotification {
    /// Flag that is true if the binding has changed since the stream was last polled
    ready: AtomicBool,

    /// The tasks that have received a 'NotReady' notification for this stream
    notify: Mutex<Vec<Task>>
}

impl<Value, Binding> BindingStream<Value, Binding>
where Binding: Bound<Value> {
    ///
    /// Creates a new stream that can be used to track values that are changing in a binding 
    ///
    pub fn new(binding: Binding) -> BindingStream<Value, Binding> {
        let ready = Arc::new(ReadyNotification { 
            ready:  AtomicBool::new(true),
            notify: Mutex::new(vec![])
        });

        let ready_lifetime = binding.when_changed(ready.clone());

        BindingStream {
            phantom_value:  PhantomData,
            binding:        binding,
            ready:          ready,
            ready_lifetime: ready_lifetime
        }
    }
}

impl ReadyNotification {
    ///
    /// Adds the current task to the list to be notified when the change arrives
    /// 
    fn notify_current_task(&self) {
        self.notify.lock().unwrap().push(task::current());
    }
}

///
/// Set our mutex to true whenever the stream becomes 'ready'
///
impl Notifiable for ReadyNotification {
    fn mark_as_changed(&self) {
        self.ready.store(true, Ordering::Release);

        {
            // Notify the tasks. These are added as side-effects to a NotReady poll result
            let mut tasks = self.notify.lock().unwrap();

            tasks.iter_mut().for_each(|task| task.notify());

            // All notified
            *tasks = vec![];
        }
    }
}

impl<Value, Binding> Stream for BindingStream<Value, Binding>
where Binding: Bound<Value> {
   type Item=Value;
   type Error=();

   fn poll(&mut self) -> Poll<Option<Value>, ()> {
        // Find out if there's a new value ready, and flag it as consumed if there is
        let ready = {
           let was_ready = self.ready.ready.load(Ordering::Acquire);
           self.ready.ready.store(false, Ordering::Release);
           was_ready
        };
        
        // If this has changed since the last time we retrieved a value, then we can return the binding
        if ready {
            // Return the result
            Ok(Ready(Some(self.binding.get())))
        } else {
            // Black magic side-effect: notify the current task when the change comes in
            self.ready.notify_current_task();

            // Result is not ready
            Ok(NotReady)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;

    use futures::executor;
    use futures::executor::{Notify, NotifyHandle};
    use futures::future::*;
    use futures::sync::oneshot;
    use std::thread::{spawn, sleep};
    use std::time::*;
    use std::time::Instant;

    struct NotifyNothing;
    impl Notify for NotifyNothing {
        fn notify(&self, _: usize) { }
    }

    #[test]
    fn stream_returns_initial_value() {
        let binding     = bind(1);
        let mut stream  = executor::spawn(BindingStream::new(binding.clone()));
        let notify      = NotifyHandle::from(&NotifyNothing);

        assert!(stream.poll_stream_notify(&notify, 0) == Ok(Ready(Some(1))));
    }

    #[test]
    fn stream_is_not_ready_after_reading() {
        let binding     = bind(1);
        let mut stream  = executor::spawn(BindingStream::new(binding.clone()));
        let notify      = NotifyHandle::from(&NotifyNothing);

        let _ = stream.poll_stream_notify(&notify, 0);
        assert!(stream.poll_stream_notify(&notify, 0) == Ok(NotReady));
    }

    #[test]
    fn stream_stays_unready_if_change_does_not_affect_value() {
        let mut binding = bind(1);
        let mut stream  = executor::spawn(BindingStream::new(binding.clone()));
        let notify      = NotifyHandle::from(&NotifyNothing);

        let _ = stream.poll_stream_notify(&notify, 0);
        let _ = stream.poll_stream_notify(&notify, 0);

        binding.set(1);
        assert!(stream.poll_stream_notify(&notify, 0) == Ok(NotReady));
    }

    #[test]
    fn stream_becomes_ready_after_change() {
        let mut binding = bind(1);
        let mut stream  = executor::spawn(BindingStream::new(binding.clone()));
        let notify      = NotifyHandle::from(&NotifyNothing);

        let _ = stream.poll_stream_notify(&notify, 0);
        let _ = stream.poll_stream_notify(&notify, 0);

        binding.set(2);
        assert!(stream.poll_stream_notify(&notify, 0) == Ok(Ready(Some(2))));
    }

    #[test]
    fn will_notify_change() {
        let mut binding = bind(1);
        let mut stream  = BindingStream::new(binding.clone());

        // As we don't return NotReady here, we won't panic with this direct call even though there's no current task
        assert!(stream.poll() == Ok(Ready(Some(1))));

        // Simple one-shot timeout that waits in a separate thread. We use this to wake things up if our stream fails to notify
        let (timeout_send, timeout_recv) = oneshot::channel::<i32>();

        spawn(move || {
            sleep(Duration::from_millis(2000));
            timeout_send.send(0).unwrap();
        });

        let timeout = timeout_recv.map_err(|_| ());

        // Create a future that will update when the timeout runs out or when the binding notifies
        // If both notify, then putting the timeout first here means it'll probably be the first returned
        let mut timeout_or_stream = executor::spawn(timeout
            .select(stream.into_future()
                .map(|(x, _)| x.unwrap())
                .map_err(|_| ()))
            .map_err(|_| ()));

        // Start a background thread that will update the binding first
        spawn(move || {
            sleep(Duration::from_millis(100));
            binding.set(2);
        });

        // First notification should be from the binding
        let start = Instant::now();
        assert!(timeout_or_stream.wait_future().unwrap().0 == 2);

        // Also as the binding was updated after 100ms, the total time shold be <1000ms
        // (Needed because futures does poll again during the select and the binding is updated there)
        assert!(start.elapsed() < Duration::from_millis(1000));
    }
}

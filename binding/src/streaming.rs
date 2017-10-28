use super::traits::*;

use std::sync::{Arc, Mutex};
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter};
use std::fmt;

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

impl<Value, Binding> Debug for BindingStream<Value, Binding> where Binding: Bound<Value> {
    fn fmt<'a>(&self, formatter: &mut Formatter<'a>) -> Result<(), fmt::Error> {
        // So we can unwrap futures involving this easily without it moaning about errors
        formatter.write_str("BindingStream")
    }
}

///
/// Used as the notification target
///
struct ReadyNotification {
    /// Flag that is true if the binding has changed since the stream was last polled
    ready: Mutex<bool>,

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
            ready:  Mutex::new(true),
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
        {
            // Mark as ready
            let mut ready_flag = self.ready.lock().unwrap();
            *ready_flag = true;
        }

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

   fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // Find out if there's a new value ready, and flag it as consumed if there is
        let ready = {
           let mut is_ready = self.ready.ready.lock().unwrap();
           let was_ready    = *is_ready;

           *is_ready = false;

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
    extern crate futures_cpupool;

    use super::*;
    use super::super::*;

    use futures::future::*;
    use self::futures_cpupool::*;
    use std::thread::{spawn, sleep};
    use std::time::*;
    use std::result::*;
    use std::time::Instant;

    #[test]
    fn stream_returns_initial_value() {
        let binding     = bind(1);
        let mut stream  = BindingStream::new(binding.clone()).peekable();

        assert!(stream.peek() == Ok(Ready(Some(&1))));
    }

    #[test]
    fn stream_is_not_ready_after_reading() {
        let binding     = bind(1);
        let mut stream  = BindingStream::new(binding.clone()).peekable();

        stream = stream.into_future().wait().unwrap().1;
        assert!(stream.peek() == Ok(NotReady));
    }

    #[test]
    fn stream_stays_unready_if_change_does_not_affect_value() {
        let mut binding = bind(1);
        let mut stream  = BindingStream::new(binding.clone());

        let _ = stream.poll();
        let _ = stream.poll();

        binding.set(1);
        assert!(stream.poll() == Ok(NotReady));
    }

    #[test]
    fn stream_becomes_ready_after_change() {
        let mut binding = bind(1);
        let mut stream  = BindingStream::new(binding.clone());

        let _ = stream.poll();
        let _ = stream.poll();

        binding.set(2);
        assert!(stream.poll() == Ok(Ready(Some(2))));
    }

    #[test]
    fn will_wait_for_change() {
        let mut binding = bind(1);
        let mut stream  = BindingStream::new(binding.clone());

        assert!(stream.poll() == Ok(Ready(Some(1))));

        let pool    = CpuPool::new(2);

        // Timeouts are in the full tokio library but we really don't want to be bringing that massive thing in just for this one test
        let timeout = pool.spawn_fn(|| {
            sleep(Duration::from_millis(2000));

            // futures always have an error in them, which breaks rust's type inference in an annoying way
            let res: Result<i32, ()> = Ok(0);
            res
        });

        // Create a future that will update when the timeout runs out or when the binding notifies
        // If both notify, then putting the timeout first here means it'll probably be the first returned
        let timeout_or_stream = timeout
            .select(stream.into_future()
                .map(|(x, _)| x.unwrap())
                .map_err(|_| ()))
            .map_err(|_| ());

        // Start a background thread that will update the binding first
        spawn(move || {
            sleep(Duration::from_millis(100));
            binding.set(2);
        });

        // First notification should be from the binding
        let start = Instant::now();
        assert!(timeout_or_stream.wait().unwrap().0 == 2);

        // Also as the binding was updated after 100ms, the total time shold be <1000ms
        // (Needed because futures does poll again during the select and the binding is updated there)
        assert!(start.elapsed() < Duration::from_millis(1000));
    }
}

use super::traits::*;

use std::sync::{Arc, Mutex};
use std::marker::PhantomData;
use futures::{Stream, Poll};
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
    ready: Mutex<bool>
}

impl<Value, Binding> BindingStream<Value, Binding>
where Binding: Bound<Value> {
    ///
    /// Creates a new stream that can be used to track values that are changing in a binding 
    ///
    pub fn new(binding: Binding) -> BindingStream<Value, Binding> {
        let ready = Arc::new(ReadyNotification { 
            ready: Mutex::new(true)
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

///
/// Set our mutex to true whenever the stream becomes 'ready'
///
impl Notifiable for ReadyNotification {
    fn mark_as_changed(&self) {
        (*self.ready.lock().unwrap()) = true;
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
            Ok(Ready(Some(self.binding.get())))
        } else {
            Ok(NotReady)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;

    #[test]
    fn stream_returns_initial_value() {
        let binding     = bind(1);
        let mut stream  = BindingStream::new(binding.clone());

        assert!(stream.poll() == Ok(Ready(Some(1))));
    }

    #[test]
    fn stream_is_not_ready_after_reading() {
        let binding     = bind(1);
        let mut stream  = BindingStream::new(binding.clone());

        let _ = stream.poll();
        assert!(stream.poll() == Ok(NotReady));
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
}

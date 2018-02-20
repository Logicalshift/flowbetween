use super::traits::*;
use super::notify_fn::*;

use futures::*;
use futures::task;
use futures::task::Task;

use std::sync::*;
use std::marker::PhantomData;

///
/// The state of the binding for a follow stream
/// 
#[derive(Copy, Clone)]
enum FollowState {
    Unchanged,
    Changed
}

///
/// Core data structures for a follow stream
/// 
struct FollowCore<TValue, Binding: Bound<TValue>> {
    /// Changed if the binidng value has changed, or Unchanged if it is not changed
    state: FollowState,

    /// What to notify when this item is changed
    notify: Option<Task>,

    /// The binding that this is following
    binding: Binding,

    /// Value is stored in the binding
    value: PhantomData<TValue>
}

///
/// Stream that follows the values of a binding
/// 
pub struct FollowStream<TValue, Binding: Bound<TValue>> {
    /// The core of this future
    core: Arc<Mutex<FollowCore<TValue, Binding>>>,

    /// Lifetime of the watcher
    watcher: Box<Releasable>,
}

impl<TValue, Binding: Bound<TValue>> Stream for FollowStream<TValue, Binding> {
    type Item   = TValue;
    type Error  = ();

    fn poll(&mut self) -> Poll<Option<TValue>, ()> {
        let mut core = self.core.lock().unwrap();

        match core.state {
            FollowState::Unchanged => {
                // Wake this future when changed
                core.notify = Some(task::current());
                Ok(Async::NotReady)
            },

            FollowState::Changed => {
                // Value has changed since we were last notified: return the changed value
                core.state = FollowState::Unchanged;
                Ok(Async::Ready(Some(core.binding.get())))
            }
        }
    }
}

///
/// Creates a stream from a binding
/// 
pub fn follow<TValue: 'static+Send, Binding: 'static+Bound<TValue>>(binding: Binding) -> FollowStream<TValue, Binding> {
    // Generate the initial core
    let core = FollowCore {
        state:      FollowState::Changed,
        notify:     None,
        binding:    binding,
        value:      PhantomData
    };

    // Notify whenever the binding changes
    let core        = Arc::new(Mutex::new(core));
    let weak_core   = Arc::downgrade(&core);
    let watcher     = core.lock().unwrap().binding.when_changed(notify(move || {
        if let Some(core) = weak_core.upgrade() {
            let mut core = core.lock().unwrap();

            core.state = FollowState::Changed;

            if let Some(notify) = core.notify.take() {
                notify.notify()
            }
        }
    }));

    // Create the stream
    FollowStream {
        core:       core,
        watcher:    watcher
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;

    use futures::executor;
    use futures::executor::Notify;
    use futures::executor::NotifyHandle;

    use std::thread;
    use std::time::Duration;

    struct NotifyNothing;
    impl Notify for NotifyNothing {
        fn notify(&self, _: usize) { }
    }

    #[test]
    fn follow_stream_has_initial_value() {
        let binding     = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::spawn(follow(bind_ref));

        assert!(stream.wait_stream() == Some(Ok(1)));
    }

    #[test]
    fn follow_stream_updates() {
        let mut binding = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::spawn(follow(bind_ref));

        assert!(stream.wait_stream() == Some(Ok(1)));
        binding.set(2);
        assert!(stream.wait_stream() == Some(Ok(2)));
    }

    #[test]
    fn stream_is_unready_after_first_read() {
        let binding     = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::spawn(follow(bind_ref));

        assert!(stream.wait_stream() == Some(Ok(1)));
        assert!(stream.poll_stream_notify(&NotifyHandle::from(&NotifyNothing), 1) == Ok(Async::NotReady));
    }

    #[test]
    fn stream_is_immediate_ready_after_write() {
        let mut binding = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::spawn(follow(bind_ref));

        assert!(stream.wait_stream() == Some(Ok(1)));
        binding.set(2);
        assert!(stream.poll_stream_notify(&NotifyHandle::from(&NotifyNothing), 1) == Ok(Async::Ready(Some(2))));
    }

    #[test]
    fn will_wake_when_binding_is_updated() {
        let mut binding = bind(1);
        let bind_ref    = BindRef::from(binding.clone());
        let mut stream  = executor::spawn(follow(bind_ref));

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            binding.set(2);
        });

        assert!(stream.wait_stream() == Some(Ok(1)));
        assert!(stream.wait_stream() == Some(Ok(2)));
    }
}

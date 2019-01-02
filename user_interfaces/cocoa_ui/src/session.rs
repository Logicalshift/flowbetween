use flo_cocoa_pipe::*;

use futures::*;
use futures::executor;
use futures::executor::Spawn;

use cocoa::base::nil;
use objc::rc::*;
use objc::runtime::*;

use std::sync::*;
use std::collections::HashMap;

///
/// Basis class for a Cocoa session
///
pub struct CocoaSession {
    /// Reference to the FloControl we'll relay the stream via
    target_object: StrongPtr,

    /// Maps IDs to windows
    windows: HashMap<usize, StrongPtr>,

    /// Maps IDs to views
    views: HashMap<usize, StrongPtr>,

    /// The stream of actions for this session (or None if we aren't monitoring for actions)
    actions: Option<Spawn<Box<dyn Stream<Item=AppAction, Error=()>+Send>>>
}

///
/// Object to notify when it's time to drain the action stream again
///
struct CocoaSessionNotify {
    notify_object: Mutex<NotifyRef>
}

///
/// Reference to an object to be notified
///
struct NotifyRef {
    target_object: WeakPtr
}

impl CocoaSession {
    ///
    /// Creates a new CocoaSession
    ///
    pub fn new(obj: &StrongPtr) -> CocoaSession {
        CocoaSession {
            target_object:  obj.clone(),
            windows:        HashMap::new(),
            views:          HashMap::new(),
            actions:        None
        }
    }

    ///
    /// Listens for actions from the specified stream
    ///
    pub fn listen_to<Actions>(&mut self, actions: Actions)
    where Actions: 'static+Send+Stream<Item=AppAction, Error=()> {
        // Spawn the actions stream
        self.actions = Some(executor::spawn(Box::new(actions)));

        unsafe {
            // Wake up the object on the main thread
            msg_send![class!(NSObject), performSelectorOnMainThread: sel!(actionStreamReady) withObject: self.target_object.clone() waitUntilDone: NO];
        }
    }

    ///
    /// Drains any pending messages from the actions stream
    ///
    pub fn drain_action_stream(&mut self) {
        // Create the object to notify when there's an update
        let notify = Arc::new(CocoaSessionNotify::new(&self.target_object));

        // Drain the stream until it's empty or it blocks
        loop {
            let next = self.actions
                .as_mut()
                .map(|actions| actions.poll_stream_notify(&notify, 0))
                .unwrap_or_else(|| Ok(Async::NotReady));

            match next {
                Ok(Async::NotReady)     => { break; }
                Ok(Async::Ready(None))  => {
                    // Session has finished
                    break;
                }

                Ok(Async::Ready(Some(action))) => {
                    // Perform the action
                    self.dispatch_app_action(action);
                }

                Err(_) => {
                    // Action stream should never produce any errors
                    unimplemented!("Action stream should never produce any errors")
                }
            }
        }
    }

    ///
    /// Performs an application action on this object
    ///
    pub fn dispatch_app_action(&mut self, action: AppAction) {
        use self::AppAction::*;

        match action {
            CreateWindow(window_id)             => { self.create_window(window_id); }
            Window(window_id, window_action)    => { self.windows.get(&window_id).map(|window| self.dispatch_window_action(window, window_action)); }
            CreateView(view_id, view_type)      => { self.create_view(view_id, view_type); },
            View(view_id, view_action)          => { self.views.get(&view_id).map(|view| self.dispatch_view_action(view, view_action)); }
        }
    }

    ///
    /// Creates a new window and assigns the specified ID to it
    ///
    fn create_window(&mut self, new_window_id: usize) {

    }

    ///
    /// Dispatches an action to the specified window
    ///
    fn dispatch_window_action(&self, window: &StrongPtr, action: WindowAction) {

    }

    ///
    /// Creates a new view and assigns the specified ID to it
    ///
    fn create_view(&mut self, new_view_id: usize, view_type: ViewType) {

    }

    ///
    /// Dispatches an action to the specified view
    ///
    fn dispatch_view_action(&self, window: &StrongPtr, action: ViewAction) {

    }
}

/// WeakPtr is not Send because Object is not Send... but we need to be able to send objective-C objects between threads so
/// we can schedule on the main thread and they are thread-safe at least in objective C itself, so let's assume this is
/// an oversight for now.
unsafe impl Send for CocoaSession { }
unsafe impl Send for NotifyRef { }

impl CocoaSessionNotify {
    ///
    /// Creates a notifier for the specified object
    ///
    pub fn new(obj: &StrongPtr) -> CocoaSessionNotify {
        CocoaSessionNotify {
            notify_object: Mutex::new(
                NotifyRef { target_object: obj.weak() }
            )
        }
    }
}

impl executor::Notify for CocoaSessionNotify {
    fn notify(&self, _: usize) {
        // Load the target object
        let target_object = self.notify_object.lock().unwrap();
        let target_object = target_object.target_object.load();

        // If it still exists, send the message to the object on the main thread
        unsafe {
            if *target_object != nil {
                msg_send![class!(NSObject), performSelectorOnMainThread: sel!(actionStreamReady) withObject: target_object waitUntilDone: NO];
            }
        }
    }
}
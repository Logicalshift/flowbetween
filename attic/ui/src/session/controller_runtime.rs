use crate::controller::*;

use flo_binding::*;

use futures::prelude::*;
use futures::future;
use futures::future::{BoxFuture};
use futures::task;
use futures::task::{Context, Poll, ArcWake, Waker};

use std::mem;
use std::sync::*;
use std::collections::{HashSet, HashMap};

///
/// Represents the parts of the runtime for a controller that have changes pending
///
struct WakeState {
    subcontrollers_stream_awake:    Mutex<bool>,
    runtime_awake:                  Mutex<bool>,
    subcontroller_awake:            Mutex<HashSet<String>>,
    waker:                          Mutex<Option<Waker>>
}

/// Waker for the subcontrollers in a wake state
struct SubcontrollerStreamWaker(Arc<WakeState>);

/// Waker for the main runtime in a wake state
struct RuntimeAwaker(Arc<WakeState>);

/// Waker for a specific subcontroller in a wake state
struct SubcontrollerWaker(String, Arc<WakeState>);

impl WakeState {
    ///
    /// Returns true if the subcontrollers stream has awoken since the last time this was called
    ///
    fn is_subcontrollers_stream_awake(&self) -> bool { 
        let mut subcontrollers_stream_awake = self.subcontrollers_stream_awake.lock().unwrap();
        let is_awake                        = *subcontrollers_stream_awake;
        *subcontrollers_stream_awake        = false;

        is_awake
    }

    ///
    /// Returns true if the runtime future has awoken since the last time this was called
    ///
    fn is_runtime_awake(&self) -> bool {
        let mut runtime_awake   = self.runtime_awake.lock().unwrap();
        let is_awake            = *runtime_awake;
        *runtime_awake          = false;

        is_awake
    }

    ///
    /// Returns the list of subcontroller names that have been awoken since the last time this was called
    ///
    fn awake_subcontrollers(&self) -> HashSet<String> {
        mem::take(&mut *self.subcontroller_awake.lock().unwrap())
    }
}

impl ArcWake for SubcontrollerStreamWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Mark the subcontroller stream as awoken
        (*arc_self.0.subcontrollers_stream_awake.lock().unwrap()) = true;

        // Wake the 'parent' waker
        (arc_self.0.waker.lock().unwrap().take()).map(|waker| waker.wake());
    }
}

impl ArcWake for RuntimeAwaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Mark the runtime future as awoken
        (*arc_self.0.runtime_awake.lock().unwrap()) = true;

        // Wake the 'parent' waker
        (arc_self.0.waker.lock().unwrap().take()).map(|waker| waker.wake());
    }
}

impl ArcWake for SubcontrollerWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Mark the subcontroller as awoken
        let subcontroller = arc_self.0.clone();
        (*arc_self.1.subcontroller_awake.lock().unwrap()).insert(subcontroller);

        // Wake the 'parent' waker
        (arc_self.1.waker.lock().unwrap().take()).map(|waker| waker.wake());
    }
}

///
/// Returns a future that runs the runtimes of a controller and any subcontrollers it may have
///
pub fn controller_runtime(controller: Arc<dyn Controller>) -> BoxFuture<'static, ()> {
    async move {
        // The user interface is used to give the names of the subcontrollers
        let ui_stream                   = follow(controller.ui());
        let mut subcontroller_stream    = ui_stream.map(|control| control.all_controllers().into_iter().collect::<HashSet<_>>());

        // We also run the main controller runtime here, if it exists
        let mut runtime                 = controller.runtime();
        let mut controller_runtimes     = HashMap::<String, _>::new();

        // Maintain only a weak reference to the controller (the future ends when the controller is gone)
        let controller                  = Arc::downgrade(&controller);

        // Initially everything is awake except the subcontrollers
        let wake_state                  = WakeState { subcontrollers_stream_awake: Mutex::new(true), runtime_awake: Mutex::new(true), subcontroller_awake: Mutex::new(HashSet::new()), waker: Mutex::new(None) };
        let wake_state                  = Arc::new(wake_state);

        // Track the futures
        let poll_futures                = future::poll_fn(move |context| {
            // If the list of subcontrollers change, then we always poll them all
            let mut wake_all_subcontrollers = false;

            // Replace the waker in the wake state
            (*wake_state.waker.lock().unwrap()) = Some(context.waker().clone());

            // Upgrade the controller while polling (if the controller has been released the runtime finishes)
            let controller = if let Some(controller) = controller.upgrade() { controller } else { return Poll::Ready(()) };

            if wake_state.is_subcontrollers_stream_awake() {
                let waker       = task::waker(Arc::new(SubcontrollerStreamWaker(Arc::clone(&wake_state))));
                let mut context = Context::from_waker(&waker);
                let context     = &mut context;

                // Check for updates from the subcontrollers (if the UI stream has finished, the runtime finishes)
                loop {
                    match subcontroller_stream.poll_next_unpin(context) {
                        Poll::Pending                       => { break; }
                        Poll::Ready(None)                   => { return Poll::Ready(()); }
                        Poll::Ready(Some(subcontrollers))   => {
                            // Remove any runtime for a subcontroller that's no longer in the UI
                            let mut removed_controllers = vec![];

                            for existing_subcontroller_name in controller_runtimes.keys() {
                                if !subcontrollers.contains(existing_subcontroller_name)  {
                                    removed_controllers.push(existing_subcontroller_name.clone());
                                }
                            }

                            removed_controllers.into_iter().for_each(|removed_name| { controller_runtimes.remove(&removed_name); });

                            // Add new runtimes for any subcontroller that's not in the list
                            for subcontroller_name in subcontrollers.iter() {
                                if !controller_runtimes.contains_key(subcontroller_name) {
                                    let subcontroller   = controller.get_subcontroller(subcontroller_name);
                                    let runtime         = subcontroller.map(|subcontroller| controller_runtime(subcontroller));

                                    controller_runtimes.insert(subcontroller_name.clone(), runtime);
                                    wake_all_subcontrollers = true;
                                }
                            }
                        }
                    }
                }
            }

            // Poll the main runtime. A single controller can finish its runtime without finishing the entire stream
            if wake_state.is_runtime_awake() {
                let waker       = task::waker(Arc::new(RuntimeAwaker(Arc::clone(&wake_state))));
                let mut context = Context::from_waker(&waker);
                let context     = &mut context;

                if let Some(active_runtime) = &mut runtime {
                    if active_runtime.poll_unpin(context) == Poll::Ready(()) {
                        // Unset the runtime once it has finished
                        runtime = None;
                    }
                }
            }

            // Poll the subcontrollers
            let awake_subcontrollers = wake_state.awake_subcontrollers();

            for (subcontroller_name, subcontroller_runtime) in controller_runtimes.iter_mut() {
                if awake_subcontrollers.contains(subcontroller_name) || wake_all_subcontrollers {
                    let waker       = task::waker(Arc::new(SubcontrollerWaker(subcontroller_name.clone(), Arc::clone(&wake_state))));
                    let mut context = Context::from_waker(&waker);
                    let context     = &mut context;

                    if let Some(runtime) = subcontroller_runtime {
                        if runtime.poll_unpin(context) == Poll::Ready(()) {
                            // Don't poll this subcontroller further if its future finishes
                            *subcontroller_runtime = None;
                        }
                    }
                }
            }

            // Return value is pending until the controller finishes
            Poll::Pending
        });

        // Wait until the futures finish running
        poll_futures.await;
    }.boxed()
}

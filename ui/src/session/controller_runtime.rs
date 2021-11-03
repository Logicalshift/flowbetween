use crate::controller::*;

use flo_binding::*;

use futures::prelude::*;
use futures::future;
use futures::future::{BoxFuture};
use futures::task::{Poll};

use std::sync::*;
use std::collections::{HashSet, HashMap};

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

        // Track the futures
        let poll_futures                = future::poll_fn(move |context| {
            // TODO: would be more efficient to track which of these futures has signalled and only poll when it does rather than always polling everything 
            // (will poll the whole way down the subcontroller stack)

            // Upgrade the controller while polling (if the controller has been released the runtime finishes)
            let controller = if let Some(controller) = controller.upgrade() { controller } else { return Poll::Ready(()) };

            // Check for updates from the subcontrollers (if the UI stream has finished, the runtime finishes)
            match subcontroller_stream.poll_next_unpin(context) {
                Poll::Pending                       => { }
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
                        }
                    }
                }
            }

            // Poll the main runtime. A single controller can finish its runtime without finishing the entire stream
            if let Some(active_runtime) = &mut runtime {
                if active_runtime.poll_unpin(context) == Poll::Ready(()) {
                    // Unset the runtime once it has finished
                    runtime = None;
                }
            }

            // Poll the subcontrollers
            for subcontroller_runtime in controller_runtimes.values_mut() {
                if let Some(runtime) = subcontroller_runtime {
                    if runtime.poll_unpin(context) == Poll::Ready(()) {
                        // Don't poll this subcontroller further if its future finishes
                        *subcontroller_runtime = None;
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

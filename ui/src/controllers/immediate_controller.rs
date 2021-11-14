use super::controller_action::*;
use super::controller_resources::*;
use crate::image::*;
use crate::control::*;
use crate::viewmodel::*;
use crate::controller::*;
use crate::binding_canvas::*;
use super::controller_event::*;
use crate::resource_manager::*;
use crate::dynamic_viewmodel::*;

use futures::prelude::*;
use futures::future;
use futures::future::{BoxFuture};
use futures::channel::mpsc;

use flo_stream::*;

use flo_binding::*;

use std::mem;
use std::iter;
use std::sync::*;
use std::collections::{HashMap};

///
/// An immediate-mode controller implements the controller trait and runs the controller by dispatching and receiving messages
/// on a stream. This allows for writing controllers that run as streams, and also removes most of the boilerplate
/// around setting up resource managers and view models.
///
/// A future-based runtime like this also makes it easier to update the controller in the background and manage different
/// states (say when tracking mouse drags and drawing actions)
///
pub struct ImmediateController<TNewFuture> {
    /// Creates a new runtime for the controller
    make_runtime: TNewFuture,

    /// The user interface control
    ui: BindRef<Control>,

    /// The resources for this controller
    resources: ControllerResources,

    /// The core for this immediate mode controller
    core: Arc<Mutex<ImmediateControllerCore>>
}

impl<TFuture: 'static+Send+Future<Output=()>, TNewFuture: Sync+Send+Fn(ControllerEventStream, mpsc::Sender<ControllerAction>, ControllerResources) -> TFuture> ImmediateController<TNewFuture> {
    ///
    /// Creates a new immediate-mode controller. The function passed in should be able to create the runtime for the controller whenever it is 
    /// displayed on screen.
    ///
    /// The resources and the UI can be specified ahead of time to avoid any point where the controller might display an empty UI due to the
    /// runtime initialising late.
    ///
    pub fn new(resources: ControllerResources, default_ui: BindRef<Control>, create_runtime: TNewFuture) -> ImmediateController<TNewFuture> {
        // The UI defaults to an empty stream
        let initial_ui              = default_ui.get();
        let default_ui_stream       = follow(default_ui);
        let (ui_stream, ui_switch)  = switchable_stream(default_ui_stream);
        let ui                      = bind_stream(ui_stream, initial_ui, |_, new_value| new_value);

        // The event core is initially not connected anywhere
        let (event_core, _events)   = ControllerEventStream::new();

        // Create the core
        let core = ImmediateControllerCore {
            viewmodel:      Arc::new(DynamicViewModel::new()),
            ui_switch:      ui_switch,
            canvases:       Arc::clone(resources.canvases()),
            images:         Arc::clone(resources.images()),
            subcontrollers: HashMap::new(),
            event_core:     event_core
        };

        // Create the controller
        ImmediateController {
            make_runtime:   create_runtime,
            ui:             BindRef::from(ui),
            resources:      ControllerResources::new(),
            core:           Arc::new(Mutex::new(core))
        }
    }
}

impl<TFuture: 'static+Send+Future<Output=()>, TNewFuture: Sync+Send+Fn(ControllerEventStream, mpsc::Sender<ControllerAction>, ControllerResources) -> TFuture> Controller for ImmediateController<TNewFuture> {
    ///
    /// Retrieves a Control representing the UI for this controller
    ///
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    ///
    /// Retrieves the viewmodel for this controller
    ///
    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        let core        = self.core.lock().unwrap();
        let viewmodel   = Arc::clone(&core.viewmodel);
        Some(viewmodel)
    }

    ///
    /// Attempts to retrieve a sub-controller of this controller
    ///
    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
        let core            = self.core.lock().unwrap();
        let subcontroller   = core.subcontrollers.get(id).map(|subcontroller| Arc::clone(subcontroller));

        subcontroller
    }

    ///
    /// Callback for when a control associated with this controller generates an action
    ///
    fn action(&self, action_id: &str, action_data: &ActionParameter) {
        // Queue the action to any runtimes that are running
        let event_core = {
            let core = self.core.lock().unwrap();
            Arc::clone(&core.event_core)
        };

        event_core.post_events(iter::once(ControllerEvent::Action(action_id.to_string(), action_data.clone())));
    }

    ///
    /// Retrieves a resource manager containing the images used in the UI for this controller
    ///
    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        let core = self.core.lock().unwrap();
        Some(Arc::clone(&core.images))
    }

    ///
    /// Retrieves a resource manager containing the canvases used in the UI for this controller
    ///
    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> {
        let core = self.core.lock().unwrap();
        Some(Arc::clone(&core.canvases))
    }

    ///
    /// Returns a future representing the run-time for this controller
    ///
    /// This is run in sync with the main UI thread: ie, all controllers that have a future must
    /// be asleep before a tick can pass. This also provides a way for a controller to wake the
    /// run-time thread.
    ///
    fn runtime(&self) -> Option<BoxFuture<'static, ()>> {
        let (event_core, events, pending) = {
            // Fetch the core
            let mut core = self.core.lock().unwrap();

            // Replace the events for the current core (if there's another running runtime, its events will close)
            core.event_core.close();
            let (event_core, events)    = ControllerEventStream::new();
            let pending                 = core.event_core.take_pending();
            core.event_core             = Arc::clone(&event_core);

            (event_core, events, pending)
        };

        // Create a channel for processing actions
        let (send_actions, receive_actions) = mpsc::channel(100);

        // Create a future for processing the actions generated by the runtime
        let core            = Arc::clone(&self.core);
        let core            = Arc::downgrade(&core);
        let process_actions = async move {
            let mut receive_actions = receive_actions;

            // Process actions as long as they exist
            while let Some(action)  = receive_actions.next().await {
                if let Some(core) = core.upgrade() {
                    // Send this action to the core
                    let mut core = core.lock().unwrap();
                    core.send_action(action);
                } else {
                    // The core is no longer available: stop receiving future messages
                    break;
                }
            }

            // Make sure that receive_actions is dropped so that the runtime knows it can't send messages any more
            mem::drop(receive_actions);

            // Wait forever so the main runtime determines when the future is terminated
            future::pending::<()>().await;
        };

        // Dispatch the pending events from the old core to the new core (if there is one)
        event_core.post_events(pending.into_iter());

        // Start the runtime for this stream going (possibly a new copy, though usually only one runtime should be active at any one time)
        let runtime = (self.make_runtime)(events, send_actions, self.resources.clone());
        let runtime = future::select(runtime.boxed(), process_actions.boxed()).map(|_| ());

        Some(runtime.boxed())
    }

    ///
    /// Called just before an update is processed
    ///
    /// This is called for every controller every time after processing any actions
    /// that might have occurred.
    ///
    fn tick(&self) { }
}

///
/// The core state for an immediate mode controller
///
pub (crate) struct ImmediateControllerCore {
    /// The viewmodel for this controller
    viewmodel: Arc<DynamicViewModel>,

    /// Used to switch the source of UI values
    ui_switch: StreamSwitch<Control>,

    /// The canvases for this immediate mode controller
    canvases: Arc<ResourceManager<BindingCanvas>>,

    /// The images for this immediate mode controller
    images: Arc<ResourceManager<Image>>,

    /// The subcontrollers that are known for this core
    subcontrollers: HashMap<String, Arc<dyn Controller>>,

    /// Used to send events to the runtime
    event_core: Arc<Mutex<ControllerEventStreamCore>>
}

impl ImmediateControllerCore {
    ///
    /// Processes an action for the controller
    ///
    fn send_action(&mut self, action: ControllerAction) {
        use self::ControllerAction::*;

        match action {
            SetUi(control)                                  => { self.ui_switch.switch_to_stream(follow(control)); },
            SetProperty(property_name, value)               => { self.viewmodel.set_property(&property_name, value); },
            SetPropertyBinding(property_name, binding)      => { self.viewmodel.set_computed(&property_name, move || binding.get()); },
            AddSubController(controller_name, controller)   => { self.subcontrollers.insert(controller_name, controller); },
            RemoveSubController(controller_name)            => { self.subcontrollers.remove(&controller_name); },
        }
    }
}

impl Drop for ImmediateControllerCore {
    fn drop(&mut self) {
        self.event_core.close();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use futures::future::{select, Either};
    use futures::executor;
    use futures::channel::oneshot;
    use futures_timer::{Delay};

    use ::desync::*;

    use std::time::{Duration};

    #[test]
    fn set_new_ui() {
        // Create a signal channel to perform the action
        let (mut signal, update_ui) = mpsc::channel(0);
        let (finished, is_finished) = oneshot::channel();

        // Create a new controller
        let update_ui       = Arc::new(Mutex::new(Some(update_ui)));
        let finished        = Arc::new(Mutex::new(Some(finished)));

        let controller      = ImmediateController::new(ControllerResources::new(), BindRef::from(bind(Control::empty())),
            move |_events, actions, _resources| {
                let finished        = Arc::clone(&finished);
                let update_ui       = update_ui.lock().unwrap().take();
                let mut update_ui   = update_ui.expect("Update UI");
                let mut actions     = actions;

                async move {
                    update_ui.next().await;

                    // Send the action
                    actions.send(ControllerAction::SetUi(BindRef::from(bind(Control::label())))).await.ok();

                    // Yield so the action can take effect
                    Delay::new(Duration::from_millis(10)).await;

                    // Note that we're finished
                    if let Some(finished) = finished.lock().unwrap().take() {
                        finished.send(()).ok();
                    }
                }
            });

        // Run the runtime in the background
        let runtime = controller.runtime().expect("Runtime");
        let runner  = Desync::new(());
        runner.future_desync(move |_| runtime.boxed()).detach();

        // Check the initial UI
        assert!(controller.ui().get() == Control::empty());

        // Send the action and wait for it to complete
        executor::block_on(async { select(signal.send(()), Delay::new(Duration::from_secs(1))).await; });

        // Wait for the action to finish sending
        executor::block_on(async { select(is_finished, Delay::new(Duration::from_secs(1))).await; });

        // UI should be updated to be a label
        assert!(controller.ui().get() == Control::label());

        // Drop the controller first to ensure the runtime terminates
        mem::drop(controller);
    }

    #[test]
    fn send_controller_event() {
        // Create a signal channel to perform the action
        let (send_event, recv_event) = oneshot::channel();

        // Create a new controller
        let send_event      = Arc::new(Mutex::new(Some(send_event)));

        let controller      = ImmediateController::new(ControllerResources::new(), BindRef::from(bind(Control::empty())),
            move |events, _actions, _resources| {
                let send_event      = send_event.lock().unwrap().take().unwrap();
                let mut events      = events;

                async move {
                    // Wait for the next event
                    let first_event = events.next().await;

                    // Send it back to the main thread
                    send_event.send(first_event).ok();
                }
            });

        // Run the runtime in the background
        let runtime = controller.runtime().expect("Runtime");
        let runner  = Desync::new(());
        runner.future_desync(move |_| runtime.boxed()).detach();

        // Send an event to it
        controller.action("Test", &ActionParameter::VirtualScroll((1, 2), (3, 4)));

        // Wait for the event to arrive
        let event_readback = executor::block_on(async { select(recv_event, Delay::new(Duration::from_secs(1))).await });

        match event_readback {
            Either::Right(((), _))      => { assert!(false, "Timeout"); }
            Either::Left((Ok(evt), _))  => { println!("{:?}", evt); assert!(evt == Some(ControllerEvent::Action("Test".to_string(), ActionParameter::VirtualScroll((1, 2), (3, 4))))); }
            Either::Left((Err(_), _))   => { assert!(false, "Error"); }
        }

        // Drop the controller first to ensure the runtime terminates
        mem::drop(controller);
    }
}

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
use futures::future::{BoxFuture};

use flo_stream::*;

use flo_binding::*;

use std::iter;
use std::sync::*;
use std::collections::{HashMap};

///
/// A stream controller implements the controller trait and runs the controller by dispatching and receiving messages
/// on a stream. This allows for writing controllers that run as streams, and also removes most of the boilerplate
/// around setting up resource managers and view models.
///
/// A future-based runtime like this also makes it easier to update the controller in the background and manage different
/// states (say when tracking mouse drags and drawing actions)
///
pub struct StreamController<TNewFuture> {
    /// Creates a new runtime for the controller
    make_runtime: TNewFuture,

    /// The resources for this controller
    resources: ControllerResources,

    /// The core for this stream controller
    core: Arc<Mutex<StreamControllerCore>>
}

impl<TFuture: 'static+Send+Future<Output=()>, TNewFuture: Sync+Send+Fn(ControllerEventStream, ControllerResources) -> TFuture> Controller for StreamController<TNewFuture> {
    ///
    /// Retrieves a Control representing the UI for this controller
    ///
    fn ui(&self) -> BindRef<Control> {
        BindRef::from(bind(Control::empty()))
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

        // Dispatch the pending events from the old core to the new core (if there is one)
        event_core.post_events(pending.into_iter());

        // Start the runtime for this stream going (possibly a new copy, though usually only one runtime should be active at any one time)
        let runtime = (self.make_runtime)(events, self.resources.clone());

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
/// The core state for a stream controller
///
pub (crate) struct StreamControllerCore {
    /// The viewmodel for this controller
    viewmodel: Arc<DynamicViewModel>,

    /// Used to switch the source of UI values
    ui_switch: StreamSwitch<Control>,

    /// The canvases for this stream controller
    canvases: Arc<ResourceManager<BindingCanvas>>,

    /// The images for this stream controller
    images: Arc<ResourceManager<Image>>,

    /// The subcontrollers that are known for this core
    subcontrollers: HashMap<String, Arc<dyn Controller>>,

    /// Used to send events to the runtime
    event_core: Arc<Mutex<ControllerEventStreamCore>>
}

impl StreamControllerCore {
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

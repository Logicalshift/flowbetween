use super::*;
use super::core::*;
use super::event::*;
use super::update::*;
use super::event_publisher::*;
use super::update_stream::*;
use super::super::controller::*;
use super::super::user_interface::*;

use ::desync::*;
use flo_binding::*;
use flo_stream::*;
use futures::prelude::*;

use std::sync::*;

///
/// UI session provides a raw user interface implementation for a core controller
///
pub struct UiSession<CoreController: Controller> {
    /// The core controller
    controller: Arc<CoreController>,

    /// The core of the UI session
    core: Arc<Desync<UiSessionCore>>,

    /// The event publisher for this session
    event_publisher: Publisher<Vec<UiEvent>>,

    /// A releasable that tracks UI updates
    ui_update_lifetime: Mutex<Box<dyn Releasable>>
}

impl<CoreController: Controller+'static> UiSession<CoreController> {
    ///
    /// Cretes a new UI session with the specified core controller, and returns a future that will run the session
    /// (and complete once the session has completed)
    ///
    pub fn new(controller: CoreController) -> (UiSession<CoreController>, impl Future<Output=()>) {
        let controller          = Arc::new(controller);
        let core                = UiSessionCore::new(controller.clone());
        let core                = Arc::new(Desync::new(core));

        let ui_update_lifetime  = Self::track_ui_updates(Arc::clone(&core));
        let publisher           = Publisher::new(100);
        let run_loop            = ui_event_loop(Arc::downgrade(&controller), publisher.republish_weak(), Arc::downgrade(&core));

        let session             = UiSession {
            controller:         controller,
            core:               core,
            event_publisher:    publisher,
            ui_update_lifetime: Mutex::new(ui_update_lifetime)
        };

        (session, run_loop)
    }

    ///
    /// Causes wake_for_updates to be called on a core when its UI changes
    ///
    fn track_ui_updates(core: Arc<Desync<UiSessionCore>>) -> Box<dyn Releasable> {
        let update_core = Arc::clone(&core);

        core.sync(move |core| {
            let ui = core.ui_tree();

            ui.when_changed(notify(move || update_core.desync(|core| core.wake_for_updates())))
        })
    }
}

impl<CoreController: Controller> Drop for UiSession<CoreController> {
    fn drop(&mut self) {
        self.ui_update_lifetime.lock().unwrap().done();
    }
}

impl<CoreController: 'static+Controller> UserInterface<Vec<UiEvent>, Vec<UiUpdate>, ()> for UiSession<CoreController> {
    /// The type of the update stream for this UI
    type UpdateStream = UiUpdateStream;

    /// Retrieves an input event sink for this user interface
    fn get_input_sink(&self) -> WeakPublisher<Vec<UiEvent>> {
        // TODO: the 'republished' version we generate here should be a weak reference so the stream ends when the session object is freed
        // (or we need a modification to flo_stream that allows for closing existing publishers)
        self.event_publisher.republish_weak()
    }

    /// Retrieves a view onto the update stream for this user interface
    fn get_updates(&self) -> UiUpdateStream {
        let (tick, update_suspend) = self.core.sync(|core| (core.subscribe_ticks(), core.subscribe_update_suspend()));

        UiUpdateStream::new(self.controller.clone(), tick, update_suspend)
    }
}

impl<CoreController: 'static+Controller> CoreUserInterface for UiSession<CoreController> {
    type CoreController = CoreController;

    fn ui_tree(&self) -> BindRef<Control> {
        self.core.sync(|core| core.ui_tree())
    }

    fn controller(&self) -> Arc<CoreController> {
        Arc::clone(&self.controller)
    }
}

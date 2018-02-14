use super::*;
use super::core::*;
use super::event::*;
use super::update::*;
use super::event_sink::*;
use super::update_stream::*;
use super::super::controller::*;
use super::super::user_interface::*;

use desync::*;
use binding::*;

use std::sync::*;
use std::ops::Deref;

///
/// UI session provides a raw user interface implementation for a core controller
/// 
pub struct UiSession<CoreController: Controller> {
    /// The core controller
    controller: Arc<CoreController>,

    /// The core of the UI session
    core: Arc<Desync<UiSessionCore>>,

    /// A releasable that tracks UI updates
    ui_update_lifetime: Mutex<Box<Releasable>>
}

impl<CoreController: Controller+'static> UiSession<CoreController> {
    ///
    /// Cretes a new UI session with the specified core controller
    /// 
    pub fn new(controller: CoreController) -> UiSession<CoreController> {
        let controller          = Arc::new(controller);
        let core                = UiSessionCore::new(controller.clone());
        let core                = Arc::new(Desync::new(core));

        let ui_update_lifetime  = Self::track_ui_updates(Arc::clone(&core));

        UiSession {
            controller:         controller,
            core:               core,
            ui_update_lifetime: Mutex::new(ui_update_lifetime)
        }
    }

    ///
    /// Causes wake_for_updates to be called on a core when its UI changes
    /// 
    fn track_ui_updates(core: Arc<Desync<UiSessionCore>>) -> Box<Releasable> {
        let update_core = Arc::clone(&core);

        core.sync(move |core| {
            let ui = core.ui_tree();

            ui.when_changed(notify(move || update_core.async(|core| core.wake_for_updates())))
        })
    }
}

impl<CoreController: Controller> Drop for UiSession<CoreController> {
    fn drop(&mut self) {
        self.ui_update_lifetime.lock().unwrap().done();
    }
}

impl<CoreController: Controller> Deref for UiSession<CoreController> {
    type Target = Arc<CoreController>;

    fn deref(&self) -> &Arc<CoreController> {
        &self.controller
    }
}

impl<CoreController: 'static+Controller> UserInterface<UiEvent, Vec<UiUpdate>, ()> for UiSession<CoreController> {
    /// The type of the event sink for this UI
    type EventSink = UiEventSink;

    /// The type of the update stream for this UI
    type UpdateStream = UiUpdateStream;

    /// Retrieves an input event sink for this user interface
    fn get_input_sink(&self) -> UiEventSink {
        UiEventSink::new(Arc::clone(&self.controller), Arc::clone(&self.core))
    }

    /// Retrieves a view onto the update stream for this user interface
    fn get_updates(&self) -> UiUpdateStream {
        UiUpdateStream::new(self.controller.clone(), Arc::clone(&self.core))
    }
}

impl<CoreController: 'static+Controller> CoreUserInterface for UiSession<CoreController> {
    fn ui_tree(&self) -> BindRef<Control> {
        self.core.sync(|core| core.ui_tree())
    }
}

use super::core::*;
use super::event::*;
use super::update::*;
use super::super::controller::*;
use super::super::user_interface::*;

use desync::*;
use futures::sink::*;

use std::sync::*;
use std::ops::Deref;

///
/// UI session provides a raw user interface implementation for a core controller
/// 
pub struct UiSession<CoreController: Controller> {
    /// The core controller
    controller: Arc<CoreController>,

    /// The core of the UI session
    core: Arc<Desync<UiSessionCore>>
}

impl<CoreController: Controller> UiSession<CoreController> {
    ///
    /// Cretes a new UI session with the specified core controller
    /// 
    pub fn new(controller: CoreController) -> UiSession<CoreController> {
        let controller  = Arc::new(controller);
        let core        = UiSessionCore::new();

        UiSession {
            controller: controller,
            core:       Arc::new(Desync::new(core))
        }
    }
}

impl<CoreController: Controller> Deref for UiSession<CoreController> {
    type Target = CoreController;

    fn deref(&self) -> &CoreController {
        &*self.controller
    }
}

/*
impl<CoreController: Controller> UserInterface<UiEvent, UiUpdate, ()> for UiSession<CoreController> {
    /// The type of the event sink for this UI
    type EventSink: Sink<SinkItem = InputEvent, SinkError = ()>;

    /// The type of the update stream for this UI
    type UpdateStream: Stream<Item = OutputUpdate, Error = Error>;

    /// Retrieves an input event sink for this user interface
    fn get_input_sink(&self) -> Self::EventSink;

    /// Retrieves a view onto the update stream for this user interface
    fn get_updates(&self) -> Self::UpdateStream;

}*/
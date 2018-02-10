use super::state::*;
use super::super::controller::*;

use desync::*;

use std::sync::*;
use std::ops::Deref;

///
/// UI session provides a raw user interface implementation for a core controller
/// 
pub struct UiSession<CoreController: Controller> {
    /// The core controller
    controller: Arc<CoreController>,

    /// The core of the UI session
    core: Desync<UiSessionCore>
}

///
/// Core UI session structures
/// 
struct UiSessionCore {
    /// The state of the UI at the last update
    state: UiSessionState
}

impl UiSessionCore {
    pub fn new() -> UiSessionCore {
        UiSessionCore {
            state: UiSessionState::new()
        }
    }
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
            core:       Desync::new(core)
        }
    }
}

impl<CoreController: Controller> Deref for UiSession<CoreController> {
    type Target = CoreController;

    fn deref(&self) -> &CoreController {
        &*self.controller
    }
}

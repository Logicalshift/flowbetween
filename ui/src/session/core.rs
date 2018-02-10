use super::state::*;
use super::event::*;
use super::super::control::*;
use super::super::controller::*;

use binding::*;

///
/// Core UI session structures
/// 
pub struct UiSessionCore {
    /// The state of the UI at the last update
    state: UiSessionState
}

impl UiSessionCore {
    ///
    /// Creates a new UI core
    /// 
    pub fn new() -> UiSessionCore {
        UiSessionCore {
            state: UiSessionState::new()
        }
    }

    ///
    /// Dispatches an event to the specified controller
    ///  
    pub fn dispatch_event(&mut self, event: UiEvent, controller: &Controller) {
        // Send the event to the controllers
        match event {
            UiEvent::Action(controller_path, event_name, action_parameter) => {
                // Find the controller along this path
                if controller_path.len() == 0 {
                    // Straight to the root controller
                    self.dispatch_action(controller, event_name, action_parameter);
                } else {
                    // Controller along a path
                    let mut controller = controller.get_subcontroller(&controller_path[0]);

                    for controller_name in controller_path.into_iter().skip(1) {
                        controller = controller.map_or(None, move |ctrl| ctrl.get_subcontroller(&controller_name));
                    }

                    match controller {
                        Some(ref controller)    => self.dispatch_action(&**controller, event_name, action_parameter),
                        None                    => ()       // TODO: event has disappeared into the void :-(
                    }
                }
            },

            UiEvent::Tick => {
                // Send a tick to this controller
                self.dispatch_tick(controller);
            }
        }

        // It might be time to wake anything waiting on the update stream
        self.wake_for_updates();
    }

    ///
    /// Wakes things up that might be waiting for updates
    /// 
    fn wake_for_updates(&mut self) {

    }

    ///
    /// Dispatches an action to a controller
    /// 
    fn dispatch_action(&mut self, controller: &Controller, event_name: String, action_parameter: ActionParameter) {
        controller.action(&event_name, &action_parameter);
    }

    ///
    /// Sends ticks to the specified controller and all its subcontrollers
    /// 
    fn dispatch_tick(&mut self, controller: &Controller) {
        // Send ticks to the subcontrollers first
        let ui              = controller.ui().get();
        let subcontrollers  = ui.all_controllers();
        for subcontroller_name in subcontrollers {
            if let Some(subcontroller) = controller.get_subcontroller(&subcontroller_name) {
                self.dispatch_tick(&*subcontroller);
            }
        }

        // Send the tick to the controller
        controller.tick();
    }
}

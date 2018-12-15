use super::event::*;
use super::update_latch::*;
use super::super::control::*;
use super::super::controller::*;

use flo_stream::*;

use binding::*;
use itertools::*;
use futures::executor::*;

use std::mem;
use std::sync::*;
use std::collections::HashMap;

///
/// Core UI session structures
/// 
pub struct UiSessionCore {
    /// The sequential ID of the last wake for update event
    last_update_id: u64,

    /// Used to publish tick events
    tick: Spawn<Publisher<()>>,

    /// Used to temporarily suspend event processing
    suspend_updates: Spawn<Publisher<UpdateLatch>>,

    /// The UI tree for the applicaiton
    ui_tree: BindRef<Control>,

    /// Functions to be called next time the core is updated
    update_callbacks: Vec<Box<dyn FnMut(&mut UiSessionCore) -> ()+Send>>
}

impl UiSessionCore {
    ///
    /// Creates a new UI core
    /// 
    pub fn new(controller: Arc<dyn Controller>) -> UiSessionCore {
        // Assemble the UI for the controller
        let ui_tree = assemble_ui(controller);

        UiSessionCore {
            last_update_id:     0,
            ui_tree:            ui_tree,
            tick:               spawn(Publisher::new(100)),
            suspend_updates:    spawn(Publisher::new(100)),
            update_callbacks:   vec![]
        }
    }

    ///
    /// Retrieves the ID of the last update that was dispatched for this core
    /// 
    pub fn last_update_id(&self) -> u64 { self.last_update_id }

    ///
    /// Retrieves a reference to the UI tree for the whole application
    /// 
    pub fn ui_tree(&self) -> BindRef<Control> { BindRef::clone(&self.ui_tree) }

    ///
    /// Finds any events in an event list that can be combined into a single event and combines them
    /// 
    pub fn reduce_events(&mut self, events: Vec<UiEvent>) -> Vec<UiEvent> {
        // Paint events destined for the same target can all be combined
        let mut paint_events: HashMap<_, _> = events.iter()
            .filter(|evt| match evt {
                UiEvent::Action(_, _, ActionParameter::Paint(_, _)) => true,
                _ => false
            })
            .map(|evt| {
                match evt {
                    UiEvent::Action(controller, event_name, ActionParameter::Paint(device, actions)) => (controller, event_name, device, actions),
                    _ => unimplemented!()
                }
            })
            .map(|paint_ref| paint_ref.clone()) 
            .group_by(|(controller, event_name, device, _actions)| (*controller, *event_name, *device)).into_iter()
            .map(|(key, group)| (key, group.into_iter().flat_map(|(_, _, _, actions)| actions).collect::<Vec<_>>()))
            .collect();

        // Turn into a results event set
        let mut result = vec![];
        for evt in events.iter() {
            match evt {
                // Paint events are all coalesced onto the first such event
                UiEvent::Action(controller, event_name, ActionParameter::Paint(device, _)) => {
                    if let Some(actions) = paint_events.get(&(controller, event_name, device)) {
                        // Append a paint event
                        result.push(UiEvent::Action(controller.clone(), event_name.clone(), ActionParameter::Paint(device.clone(), actions.iter().map(|action| (*action).clone()).collect())))
                    }

                    // Remove from the hashmap so this paint event won't get added again
                    paint_events.remove(&(controller, event_name, device));
                },

                // Standard events are just pushed
                _ => { result.push(evt.clone()); }
            }
        }

        result
    }

    ///
    /// Dispatches an event to the specified controller
    ///  
    pub fn dispatch_event(&mut self, events: Vec<UiEvent>, controller: &dyn Controller) {
        for event in events {
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

                UiEvent::SuspendUpdates => {
                    self.suspend_updates.wait_send(UpdateLatch::Suspend).ok();
                },

                UiEvent::ResumeUpdates => {
                    self.suspend_updates.wait_send(UpdateLatch::Resume).ok();
                },

                UiEvent::Tick => {
                    // Send a tick to this controller
                    self.dispatch_tick(controller);
                }
            }
        }

        // It might be time to wake anything waiting on the update stream
        self.wake_for_updates();
    }

    ///
    /// Registers a function to be called next time the core is updated
    /// 
    pub fn on_next_update<Callback: 'static+FnOnce(&mut UiSessionCore) -> ()+Send>(&mut self, callback: Callback) {
        // FnBox is in nightly, so here's our 'boxed FnOnce' workaround
        let mut callback    = Some(callback);

        // Call the function when the next update occurs
        self.update_callbacks.push(Box::new(move |session_core| {
            // Swap out the preserved callback
            let mut call_once = None;
            mem::swap(&mut call_once, &mut callback);

            // Call it if it hasn't been called before
            if let Some(call_once) = call_once {
                call_once(session_core);
            }
        }));
    }

    ///
    /// Wakes things up that might be waiting for updates
    /// 
    pub fn wake_for_updates(&mut self) {
        // Update the last update ID
        self.last_update_id += 1;

        // Perform the callbacks
        let mut callbacks = vec![];
        mem::swap(&mut callbacks, &mut self.update_callbacks);

        for mut callback in callbacks {
            callback(self);
        }
    }

    ///
    /// Returns a subscriber for tick events
    ///
    pub fn subscribe_ticks(&mut self) -> Subscriber<()> {
        self.tick.get_mut().subscribe()
    }

    ///
    /// Returns a subscriber for update suspension events
    ///
    pub fn subscribe_update_suspend(&mut self) -> Subscriber<UpdateLatch> {
        self.suspend_updates.get_mut().subscribe()
    }

    ///
    /// Dispatches an action to a controller
    /// 
    fn dispatch_action(&mut self, controller: &dyn Controller, event_name: String, action_parameter: ActionParameter) {
        controller.action(&event_name, &action_parameter);
    }

    ///
    /// Sends ticks to the specified controller and all its subcontrollers
    /// 
    fn dispatch_tick(&mut self, controller: &dyn Controller) {
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

        self.tick.wait_send(()).ok();
    }
}

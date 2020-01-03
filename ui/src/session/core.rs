use super::event::*;
use super::super::control::*;
use super::super::controller::*;

use flo_stream::*;
use flo_binding::*;

use itertools::*;
use futures::*;
use futures::future::{BoxFuture};

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
    tick: ExpiringPublisher<()>,

    /// Used to temporarily suspend event processing
    suspend_updates: ExpiringPublisher<bool>,

    /// Number of times this has been suspended
    suspension_count: i32,

    /// True if we should send a tick just before resuming the UI
    tick_on_resume: bool,

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
            tick:               ExpiringPublisher::new(1),
            suspend_updates:    ExpiringPublisher::new(1),
            suspension_count:   0,
            tick_on_resume:     false,
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
    pub async fn dispatch_event(&mut self, events: Vec<UiEvent>, controller: &dyn Controller) {
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
                    self.suspension_count += 1;
                    self.suspend_updates.publish(self.suspension_count > 0).await;
                },

                UiEvent::ResumeUpdates => {
                    if self.suspension_count == 1 && self.tick_on_resume {
                        // If a tick occurred while updates were suspended, send it now as we're just about to release the suspension
                        self.tick_on_resume = false;
                        self.dispatch_tick(controller).await;
                    }

                    self.suspension_count -= 1;
                    self.suspend_updates.publish(self.suspension_count > 0).await;
                },

                UiEvent::Tick => {
                    if self.suspension_count <= 0 {
                        // Send a tick to this controller
                        self.dispatch_tick(controller).await;
                    } else {
                        // Send a tick when updates resume
                        self.tick_on_resume = true;
                    }
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
        self.tick.subscribe()
    }

    ///
    /// Returns a subscriber for update suspension events
    ///
    pub fn subscribe_update_suspend(&mut self) -> Subscriber<bool> {
        self.suspend_updates.subscribe()
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
    #[must_use]
    fn dispatch_tick<'a>(&'a mut self, controller: &'a dyn Controller) -> BoxFuture<'a, ()> {
        async move {
            // Send ticks to the subcontrollers first
            let ui              = controller.ui().get();
            let subcontrollers  = ui.all_controllers();
            for subcontroller_name in subcontrollers {
                if let Some(subcontroller) = controller.get_subcontroller(&subcontroller_name) {
                    self.dispatch_tick(&*subcontroller).await;
                }
            }

            // Send the tick to the controller
            controller.tick();

            self.tick.publish(()).await;
        }.boxed()
    }
}

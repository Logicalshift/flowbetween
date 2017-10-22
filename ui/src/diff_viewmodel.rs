use super::controller::*;
use super::binding::*;

use std::collections::*;
use std::sync::*;

///
/// Tracks differences in the viewmodel attached to a controller and its subtree
/// 
pub struct DiffViewModel {
    /// The controller that owns the viewmodel (if it's still live)
    controller: Weak<Controller>,
}

///
/// Watches for changes in a viewmodel
///
pub struct WatchViewModel {
    /// The controller to watch
    controller: Weak<Controller>,

    /// The subcontrollers that are being watched
    subcontroller_watchers: Vec<(String, WatchViewModel)>,

    /// Which properties have changed
    changed_properties: HashMap<String, Arc<Mutex<bool>>>,

    /// Lifetimes of the watchers that update the changed properties
    watcher_lifetimes: Vec<Box<Releasable>>
}

impl DiffViewModel {
    ///
    /// Creates a new viewmodel tracker for a particular controller
    ///
    pub fn new(controller: Arc<Controller>) -> DiffViewModel {
        DiffViewModel { controller: Arc::downgrade(&controller) }
    }

    fn watch_controller(controller: &Weak<Controller>) -> WatchViewModel {
        // By default, the things we watch are empty
        let mut subcontroller_watchers      = vec![];
        let mut watcher_lifetimes           = vec![];
        let mut changed_properties          = HashMap::new();

        if let Some(controller) = controller.upgrade() {
            // Fetch the various components of the controller
            let ui              = controller.ui().get();
            let viewmodel       = controller.get_viewmodel();
            let properties      = viewmodel.get_property_names();
            let subcontrollers  = ui.all_controllers();

            // Create a 'changed' value for each property
            changed_properties = properties.iter()
                .map(|property_name| (property_name.clone(), Arc::new(Mutex::new(false))))
                .collect();

            // Watch all of the properties in the viewmodel in order to flag a change
            watcher_lifetimes = properties.iter()
                .map(|property_name| (changed_properties[property_name].clone(), viewmodel.get_property(property_name)))
                .map(|(changed, property)| property.when_changed(notify(move || *changed.lock().unwrap() = true)))
                .collect();

            // Create watchers for any subcontrollers that might be in the UI
            subcontroller_watchers = subcontrollers.iter()
                .map(|subcontroller_name| (subcontroller_name.clone(), controller.get_subcontroller(subcontroller_name)))
                .filter(|&(ref _name, ref subcontroller)| subcontroller.is_some())
                .map(|(name, subcontroller)| (name, Self::watch_controller(&Arc::downgrade(&subcontroller.unwrap()))))
                .collect();
        }

        // Result is a new watch view model
        WatchViewModel { 
            subcontroller_watchers: subcontroller_watchers,
            changed_properties:     changed_properties,
            watcher_lifetimes:      watcher_lifetimes,

            controller: controller.clone() 
        }
    }

    ///
    /// Reads the current state of the controller and creates a watcher for any changes that
    /// might occur to it.
    ///
    pub fn watch(&self) -> WatchViewModel {
        Self::watch_controller(&self.controller)
    }
}
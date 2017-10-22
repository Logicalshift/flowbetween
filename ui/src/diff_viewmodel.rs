use super::controller::*;
use super::binding::*;
use super::viewmodel_update::*;

use std::collections::*;
use std::sync::*;

///
/// Tracks differences in the viewmodel attached to a controller and its subtree
/// 
pub struct DiffViewModel {
    /// The controller that owns the viewmodel (if it's still live)
    controller: Weak<Controller>,
}

// TODO: split this into two (one struct for watching the viewmodel for a single controller
// and one for watching the whole tree)

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

    ///
    /// Reads the current state of the controller and creates a watcher for any changes that
    /// might occur to it.
    ///
    pub fn watch(&self) -> WatchViewModel {
        WatchViewModel::watch_controller(&self.controller)
    }
}

impl WatchViewModel {
    ///
    /// Creates a new watcher that watches the contents of a controller
    ///
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
    /// Retrieves the updates for the viewmodel alone
    ///
    pub fn get_local_updates(&self) -> Option<ViewModelUpdate> {
        // TODO: cycle the 'updates' for a new set so we can use this over and over?
        // Need to lock against changes to do that, which is tricky with the current design

        if let Some(controller) = self.controller.upgrade() {
            // Get the current state of the viewmodel
            let viewmodel                   = controller.get_viewmodel();
            let properties: HashSet<String> = viewmodel.get_property_names().into_iter().collect();

            // Find the changed properties; a property that is no longer in the view model cannot be changed
            let changed_properties          = self.changed_properties.iter()
                .filter(|&(ref name, ref _is_changed)| properties.contains(*name))
                .filter(|&(ref _name, ref is_changed)| *is_changed.lock().unwrap())
                .map(|(name, _is_changed)| name.clone());

            // Find the new properties: properties that aren't in the existing hash set
            let existing_properties: HashSet<String>    = self.changed_properties.keys().map(|name| name.clone()).collect();
            let new_properties                          = properties.iter()
                .filter(|property| !existing_properties.contains(*property))
                .map(|name| name.clone());

            // This is the list of properties and values to store in the result
            let properties_and_values = changed_properties.chain(new_properties)
                .map(|property_name| (property_name.clone(), viewmodel.get_property(&property_name).get()))
                .collect();

            // This is the list of updates
            Some(ViewModelUpdate::new(vec![], properties_and_values))
        } else {
            // Controller has been released since this was made
            None
        }
    }

    ///
    /// Finds all of the updates for this viewmodel
    ///
    pub fn get_updates(&self) -> Vec<ViewModelUpdate> {
        let mut all_updates = vec![];

        // Get the updates that apply to the root controller
        if let Some(our_updates) = self.get_local_updates() {
            all_updates.push(our_updates);
        }

        // Get the updates that apply to each subcontroller
        self.subcontroller_watchers.iter().for_each(|&(ref name, ref watcher)| {
            let mut updates = watcher.get_updates();
            updates.iter_mut().for_each(|ref mut update| update.add_to_start_of_path(name.clone()));

            all_updates.extend(updates);
        });

        // Return all the updates we found
        all_updates
    }
}
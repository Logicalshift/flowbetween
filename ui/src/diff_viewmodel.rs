use binding::*;

use super::property::*;
use super::controller::*;
use super::viewmodel_update::*;

use std::collections::{HashSet, HashMap};
use std::sync::*;

// TODO: needs improvement
//  * The 'watch a viewmodel' and 'watch the controller tree' functions are lumped together here and should be separate
//  * Recreating the watch every time rather than re-using the existing watchers where we can is inefficient
//  * DiffViewModel doesn't really do all that much at the moment
//  * This kind of feels like something that could be done with the futures library
//  * Watching each item individually won't work well if the viewmodel gets complicated (though the
//    current design discourages that)

///
/// Tracks differences in the viewmodel attached to a controller and its subtree
/// 
pub struct DiffViewModel {
    /// The controller that owns the viewmodel (if it's still live)
    controller: Weak<Controller>,
}

// TODO: split this into two (one struct for watching the viewmodel for a single controller
// and one for watching the whole tree). The DiffViewModel might be good for moderating this?

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

    ///
    /// Reads the updates from a WatchViewModel and create a new one that will see future updates
    ///
    pub fn rotate_watch(&self, last_watch: WatchViewModel) -> (Vec<ViewModelUpdate>, WatchViewModel) {
        // TODO: we might see the same update twice here as this version doesn't 'queue' updates that
        // occur while we're reading the original. This should be harmless, however (worst case is
        // getting a value that updates to the same value)

        // TODO 2: resetting an existing view model will be faster as we only need reset existing flags
        // and add/remove watches for items that no longer exist in the viewmodel. It's more
        // complicated, though (hm, also need to handle the set of controllers changing)
        let next_watch  = self.watch();
        let updates     = last_watch.get_updates();

        (updates, next_watch)
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
            let properties_and_values: Vec<(String, PropertyValue)> = changed_properties.chain(new_properties)
                .map(|property_name| (property_name.clone(), viewmodel.get_property(&property_name).get()))
                .collect();

            // This is the list of updates
            if properties_and_values.len() > 0 {
                Some(ViewModelUpdate::new(vec![], properties_and_values))
            } else {
                None
            }
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
            updates.iter_mut().for_each(|update| update.add_to_start_of_path(name.clone()));

            all_updates.extend(updates);
        });

        // Return all the updates we found
        all_updates
    }
}

impl Drop for WatchViewModel {
    fn drop(&mut self) {
        self.watcher_lifetimes.iter_mut().for_each(|lifetime| lifetime.done());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use super::super::control::*;
    use super::super::viewmodel::*;
    use super::super::dynamic_viewmodel::*;

    ///
    /// A controller that does nothing
    ///
    pub struct DynamicController {
        controls: Arc<Binding<Control>>,
        view_model: Arc<DynamicViewModel>,
        subcontrollers: HashMap<String, Arc<DynamicController>>
    }

    impl DynamicController {
        pub fn new() -> DynamicController {
            DynamicController { 
                controls:       Arc::new(bind(Control::empty())),
                view_model:     Arc::new(DynamicViewModel::new()),
                subcontrollers: HashMap::new()
            }
        }

        pub fn set_controls(&mut self, new_control: Control) {
            (*self.controls).clone().set(new_control);
        }

        pub fn add_subcontroller(&mut self, name: String) {
            self.subcontrollers.insert(name, Arc::new(DynamicController::new()));
        }
    }

    impl Controller for DynamicController {
        fn ui(&self) -> Arc<Bound<Control>> {
            self.controls.clone()
        }

        fn get_subcontroller(&self, id: &str) -> Option<Arc<Controller>> {
            let res = self.subcontrollers.get(id).map(|x| x.clone());

            if let Some(res) = res {
                Some(res)
            } else {
                None
            }
        }

        fn get_viewmodel(&self) -> Arc<ViewModel> {
            self.view_model.clone()
        }
    }

    #[test]
    fn initially_no_changes() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().set_property("Test", PropertyValue::Int(1));

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        assert!(watcher.get_updates() == vec![]);
    }

    #[test]
    fn changes_are_picked_up() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().set_property("Test", PropertyValue::Int(1));

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        controller.get_viewmodel().set_property("Test", PropertyValue::Int(2));

        assert!(watcher.get_updates() == vec![ViewModelUpdate::new(vec![], vec![("Test".to_string(), PropertyValue::Int(2))])]);
    }

    #[test]
    fn new_values_are_picked_up() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().set_property("Test", PropertyValue::Int(1));

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        controller.get_viewmodel().set_property("NewValue", PropertyValue::Int(2));

        assert!(watcher.get_updates() == vec![ViewModelUpdate::new(vec![], vec![("NewValue".to_string(), PropertyValue::Int(2))])]);
    }

    #[test]
    fn new_values_are_picked_up_alongside_changes() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().set_property("Test", PropertyValue::Int(1));

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        controller.get_viewmodel().set_property("Test", PropertyValue::Int(2));
        controller.get_viewmodel().set_property("NewValue", PropertyValue::Int(3));

        assert!(watcher.get_updates() == vec![ViewModelUpdate::new(vec![], vec![("Test".to_string(), PropertyValue::Int(2)), ("NewValue".to_string(), PropertyValue::Int(3))])]);
    }

    #[test]
    fn subcontroller_changes_are_picked_up() {
        let mut controller = DynamicController::new();
        controller.set_controls(Control::container().with_controller("Subcontroller"));
        controller.add_subcontroller("Subcontroller".to_string());

        let subcontroller = controller.get_subcontroller("Subcontroller").unwrap();
        subcontroller.get_viewmodel().set_property("Test", PropertyValue::Int(1));

        let controller = Arc::new(controller);

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        subcontroller.get_viewmodel().set_property("Test", PropertyValue::Int(2));

        let updates = watcher.get_updates();

        assert!(updates.len() == 1);
        assert!(updates[0].controller_path() == &vec!["Subcontroller".to_string()]);
        assert!(updates[0].updates() == &vec![("Test".to_string(), PropertyValue::Int(2))]);
    }

    // TODO: detects new controller
    // TODO: detects removed controller
}

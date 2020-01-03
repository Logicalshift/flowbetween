use flo_binding::*;

use super::viewmodel::*;
use super::controller::*;
use super::viewmodel_update::*;

use std::collections::{HashSet, HashMap};
use std::sync::*;

lazy_static! {
    // View model used when there is no view model
    pub static ref NULL_VIEW_MODEL: Arc<NullViewModel> = Arc::new(NullViewModel::new());
}

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
    controller: Weak<dyn Controller>,
}

// TODO: split this into two (one struct for watching the viewmodel for a single controller
// and one for watching the whole tree). The DiffViewModel might be good for moderating this?

///
/// Watches for changes in a viewmodel
///
pub struct WatchViewModel {
    /// The subcontrollers that were watched by this call
    subcontrollers: Vec<String>,

    /// The controller to watch
    controller: Weak<dyn Controller>,

    /// The subcontrollers that are being watched
    subcontroller_watchers: Vec<(String, WatchViewModel)>,

    /// Which properties have changed
    changed_properties: HashMap<String, Arc<Mutex<bool>>>,

    /// Lifetimes of the watchers that update the changed properties
    watcher_lifetimes: Vec<Box<dyn Releasable>>
}

impl DiffViewModel {
    ///
    /// Creates a new viewmodel tracker for a particular controller
    ///
    pub fn new(controller: Arc<dyn Controller>) -> DiffViewModel {
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
    fn watch_controller(controller: &Weak<dyn Controller>) -> WatchViewModel {
        // By default, the things we watch are empty
        let mut subcontroller_watchers      = vec![];
        let mut watcher_lifetimes           = vec![];
        let mut subcontrollers              = vec![];
        let mut changed_properties          = HashMap::new();

        if let Some(controller) = controller.upgrade() {
            // Fetch the various components of the controller
            let ui              = controller.ui().get();
            let viewmodel       = controller.get_viewmodel().unwrap_or_else(|| NULL_VIEW_MODEL.clone());
            let properties      = viewmodel.get_property_names();
            subcontrollers      = ui.all_controllers();

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
            subcontrollers:         subcontrollers,
            subcontroller_watchers: subcontroller_watchers,
            changed_properties:     changed_properties,
            watcher_lifetimes:      watcher_lifetimes,

            controller:             controller.clone()
        }
    }

    ///
    /// Retrieves the updates for the viewmodel alone
    ///
    pub fn get_local_updates(&self) -> Option<ViewModelUpdate> {
        if let Some(controller) = self.controller.upgrade() {
            // Get the current state of the viewmodel
            let viewmodel               = controller.get_viewmodel().unwrap_or_else(|| NULL_VIEW_MODEL.clone());
            let properties: HashSet<_>  = viewmodel.get_property_names().into_iter().collect();

            // Find the changed properties; a property that is no longer in the view model cannot be changed
            let changed_properties      = self.changed_properties.iter()
                .filter(|&(ref name, ref _is_changed)| properties.contains(*name))
                .filter(|&(ref _name, ref is_changed)| *is_changed.lock().unwrap())
                .map(|(name, _is_changed)| name.clone());

            // Find the new properties: properties that aren't in the existing hash set
            let existing_properties: HashSet<_> = self.changed_properties.keys().map(|name| name.clone()).collect();
            let new_properties                  = properties.iter()
                .filter(|property| !existing_properties.contains(*property))
                .map(|name| name.clone());

            // This is the list of properties and values to store in the result
            let properties_and_values: Vec<_> = changed_properties.chain(new_properties)
                .map(|property_name| ViewModelChange::PropertyChanged(property_name.clone(), viewmodel.get_property(&property_name).get()))
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
    /// Retrieves any updates caused by new subcontrollers being added to the UI
    ///
    pub fn get_new_controller_updates(&self) -> Vec<ViewModelUpdate> {
        if let Some(controller) = self.controller.upgrade() {
            // Get the current set of subcontrollers
            let ui              = controller.ui().get();
            let subcontrollers  = ui.all_controllers();

            // Find any subcontrollers that were not in the list
            let existing_controllers: HashSet<_>    = self.subcontrollers.iter().collect();
            let new_controllers: Vec<_>             = subcontrollers.iter().filter(|controller| !existing_controllers.contains(controller)).collect();

            // For any new controller, the entire viewmodel is different
            let mut result = vec![];
            for controller_name in new_controllers {
                let new_controller  = controller.get_subcontroller(controller_name);

                if let Some(new_controller) = new_controller {
                    let tree_updates    = viewmodel_update_controller_tree(&*new_controller);

                    for mut update in tree_updates {
                        update.add_to_start_of_path(controller_name.clone());
                        result.push(update);
                    }
                }
            }

            result
        } else {
            // Controller has been released since this was made
            vec![]
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

        // If there are any new subcontrollers, add them to the update list
        all_updates.extend(self.get_new_controller_updates());

        // Return all the updates we found
        all_updates
    }
}

impl Drop for WatchViewModel {
    fn drop(&mut self) {
        self.watcher_lifetimes.iter_mut().for_each(|lifetime| lifetime.done());
    }
}

///
/// Returns an update for all of the keys in a particular viewmodel
///
pub fn viewmodel_update_all(controller_path: Vec<String>, viewmodel: &dyn ViewModel) -> ViewModelUpdate {
    let keys        = viewmodel.get_property_names();
    let mut updates = vec![];

    for property_name in keys.iter() {
        let value = viewmodel.get_property(&*property_name);
        updates.push(ViewModelChange::PropertyChanged((*property_name).clone(), value.get()));
    }

    return ViewModelUpdate::new(controller_path, updates);
}

///
/// Generates the updates to set the viewmodel for an entire controller tree
///
pub fn viewmodel_update_controller_tree(controller: &dyn Controller) -> Vec<ViewModelUpdate> {
    let mut result = vec![];

    // Push the controllers to the result
    // Rust could probably capture the 'result' variable in the closure exactly liek this if it were smarter
    fn add_controller_to_result(controller: &dyn Controller, path: &mut Vec<String>, result: &mut Vec<ViewModelUpdate>) {
        // Fetch the update for the viewmodel for this controller
        let viewmodel           = controller.get_viewmodel().unwrap_or_else(|| NULL_VIEW_MODEL.clone());
        let viewmodel_update    = viewmodel_update_all(path.clone(), &*viewmodel);

        // Add to the result if there are any entries in this viewmodel
        if viewmodel_update.updates().len() > 0 {
            result.push(viewmodel_update);
        }

        // Visit any subcontrollers found in this controllers UI
        let controller_ui   = controller.ui().get();
        let subcontrollers  = controller_ui.all_controllers();

        for subcontroller_name in subcontrollers.iter() {
            if let Some(subcontroller) = controller.get_subcontroller(subcontroller_name) {
                // Recursively process this subcontroller
                path.push(subcontroller_name.clone());
                add_controller_to_result(&*subcontroller, path, result);
                path.pop();
            }
        }
    }

    // Recursively add the controllers starting at the current one
    add_controller_to_result(controller, &mut vec![], &mut result);

    result
}

#[cfg(test)]
mod test {
    use super::*;

    use super::super::control::*;
    use super::super::property::*;
    use super::super::dynamic_viewmodel::*;

    use futures::*;

    ///
    /// A controller that does nothing
    ///
    pub struct DynamicController {
        controls: Arc<Binding<Control>>,
        view_model: Arc<DynamicViewModel>,
        subcontrollers: Mutex<HashMap<String, Arc<DynamicController>>>
    }

    impl DynamicController {
        pub fn new() -> DynamicController {
            DynamicController {
                controls:       Arc::new(bind(Control::empty())),
                view_model:     Arc::new(DynamicViewModel::new()),
                subcontrollers: Mutex::new(HashMap::new())
            }
        }

        pub fn set_controls(&self, new_control: Control) {
            (*self.controls).set(new_control);
        }

        pub fn add_subcontroller(&self, name: String) {
            self.subcontrollers.lock().unwrap().insert(name, Arc::new(DynamicController::new()));
        }
    }

    impl Controller for DynamicController {
        fn ui(&self) -> BindRef<Control> {
            BindRef::from_arc(Arc::clone(&self.controls))
        }

        fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
            let res = self.subcontrollers.lock().unwrap().get(id).map(|x| x.clone());

            if let Some(res) = res {
                Some(res)
            } else {
                None
            }
        }

        fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
            Some(self.view_model.clone())
        }
    }

    #[test]
    fn initially_no_changes() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        assert!(watcher.get_updates() == vec![]);
    }

    #[test]
    fn changes_are_picked_up() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        assert!(watcher.get_updates() == vec![ViewModelUpdate::new(vec![], vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))])]);
    }

    #[test]
    fn new_values_are_picked_up() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        controller.get_viewmodel().unwrap().set_property("NewValue", PropertyValue::Int(2));

        assert!(watcher.get_updates() == vec![ViewModelUpdate::new(vec![], vec![ViewModelChange::PropertyChanged("NewValue".to_string(), PropertyValue::Int(2))])]);
    }

    #[test]
    fn new_values_are_picked_up_alongside_changes() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));
        controller.get_viewmodel().unwrap().set_property("NewValue", PropertyValue::Int(3));

        assert!(watcher.get_updates() == vec![ViewModelUpdate::new(vec![], vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2)), ViewModelChange::PropertyChanged("NewValue".to_string(), PropertyValue::Int(3))])]);
    }

    #[test]
    fn subcontroller_changes_are_picked_up() {
        let controller = DynamicController::new();
        controller.set_controls(Control::container().with_controller("Subcontroller"));
        controller.add_subcontroller("Subcontroller".to_string());

        let subcontroller = controller.get_subcontroller("Subcontroller").unwrap();
        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let controller = Arc::new(controller);

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        let updates = watcher.get_updates();

        assert!(updates.len() == 1);
        assert!(updates[0].controller_path() == &vec!["Subcontroller".to_string()]);
        assert!(updates[0].updates() == &vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))]);
    }

    #[test]
    fn new_controller_is_picked_up() {
        let controller = DynamicController::new();
        controller.set_controls(Control::container());

        let controller = Arc::new(controller);

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        let updates = watcher.get_updates();
        assert!(updates.len() == 0);

        controller.set_controls(Control::container().with_controller("Subcontroller"));
        controller.add_subcontroller("Subcontroller".to_string());
        let subcontroller = controller.get_subcontroller("Subcontroller").unwrap();

        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        let updates = watcher.get_updates();

        assert!(updates.len() == 1);
        assert!(updates[0].controller_path() == &vec!["Subcontroller".to_string()]);
        assert!(updates[0].updates() == &vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))]);
    }

    #[test]
    fn changes_after_new_controller_are_picked_up() {
        let controller = DynamicController::new();
        controller.set_controls(Control::container());

        let controller = Arc::new(controller);

        let diff_viewmodel  = DiffViewModel::new(controller.clone());
        let watcher         = diff_viewmodel.watch();

        let updates = watcher.get_updates();
        assert!(updates.len() == 0);

        controller.set_controls(Control::container().with_controller("Subcontroller"));
        controller.add_subcontroller("Subcontroller".to_string());
        let subcontroller = controller.get_subcontroller("Subcontroller").unwrap();

        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        let (_updates, watcher) = diff_viewmodel.rotate_watch(watcher);

        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(3));
        let updates = watcher.get_updates();

        assert!(updates.len() == 1);
        assert!(updates[0].controller_path() == &vec!["Subcontroller".to_string()]);
        assert!(updates[0].updates() == &vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(3))]);
    }

    struct TestViewModel;

    struct TestController {
        model_controler: Arc<ModelController>,
        view_model: Arc<NullViewModel>
    }

    struct ModelController {
        view_model: Arc<TestViewModel>
    }

    impl TestController {
        pub fn new() -> TestController {
            TestController {
                model_controler: Arc::new(ModelController::new()),
                view_model: Arc::new(NullViewModel::new())
            }
        }
    }

    impl ModelController {
        pub fn new() -> ModelController {
            ModelController { view_model: Arc::new(TestViewModel) }
        }
    }

    impl Controller for TestController {
        fn ui(&self) -> BindRef<Control> {
            BindRef::from(bind(Control::container().with(vec![
                Control::empty().with_controller("Model1"),
                Control::empty().with_controller("Model2")
            ])))
        }

        fn get_subcontroller(&self, _id: &str) -> Option<Arc<dyn Controller>> {
            Some(self.model_controler.clone())
        }

        fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
            Some(self.view_model.clone())
        }
    }

    impl Controller for ModelController {
        fn ui(&self) -> BindRef<Control> {
            BindRef::from(bind(Control::label()))
        }

        fn get_subcontroller(&self, _id: &str) -> Option<Arc<dyn Controller>> {
            None
        }

        fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
            Some(self.view_model.clone())
        }
    }

    impl ViewModel for TestViewModel {
        fn get_property(&self, property_name: &str) -> BindRef<PropertyValue> {
            BindRef::from(bind(PropertyValue::String(property_name.to_string())))
        }

        fn set_property(&self, _property_name: &str, _new_value: PropertyValue) {
        }

        fn get_property_names(&self) -> Vec<String> {
            vec![ "Test1".to_string(), "Test2".to_string(), "Test3".to_string() ]
        }

        fn get_updates(&self) -> Box<dyn Stream<Item=ViewModelChange, Error=()>+Send> {
            unimplemented!()
        }
    }

    #[test]
    pub fn can_generate_viewmodel_update_all() {
        let viewmodel   = TestViewModel;
        let update      = viewmodel_update_all(vec!["Test".to_string(), "Path".to_string()], &viewmodel);

        assert!(update.controller_path() == &vec!["Test".to_string(), "Path".to_string()]);
        assert!(update.updates() == &vec![
            ViewModelChange::PropertyChanged("Test1".to_string(), PropertyValue::String("Test1".to_string())),
            ViewModelChange::PropertyChanged("Test2".to_string(), PropertyValue::String("Test2".to_string())),
            ViewModelChange::PropertyChanged("Test3".to_string(), PropertyValue::String("Test3".to_string())),
        ]);
    }

    #[test]
    pub fn can_generate_controller_update_all() {
        let controller  = Arc::new(TestController::new());
        let update      = viewmodel_update_controller_tree(&*controller);

        assert!(update.len() == 2);

        assert!(update[0].controller_path() == &vec!["Model1".to_string()]);
        assert!(update[0].updates() == &vec![
            ViewModelChange::PropertyChanged("Test1".to_string(), PropertyValue::String("Test1".to_string())),
            ViewModelChange::PropertyChanged("Test2".to_string(), PropertyValue::String("Test2".to_string())),
            ViewModelChange::PropertyChanged("Test3".to_string(), PropertyValue::String("Test3".to_string())),
        ]);

        assert!(update[1].controller_path() == &vec!["Model2".to_string()]);
        assert!(update[1].updates() == &vec![
            ViewModelChange::PropertyChanged("Test1".to_string(), PropertyValue::String("Test1".to_string())),
            ViewModelChange::PropertyChanged("Test2".to_string(), PropertyValue::String("Test2".to_string())),
            ViewModelChange::PropertyChanged("Test3".to_string(), PropertyValue::String("Test3".to_string())),
        ]);
    }

    // TODO: detects removed controller
}

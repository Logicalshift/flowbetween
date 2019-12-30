use super::super::control::*;
use super::super::viewmodel::*;
use super::super::controller::*;
use super::super::viewmodel_update::*;

use flo_binding::*;

use futures::*;
use futures::stream;
use futures::stream::{BoxStream};
use futures::task::{Poll, Context};

use std::iter;
use std::pin::*;
use std::sync::*;
use std::collections::{HashMap, VecDeque};

///
/// A stream of updates from the viewmodels for a controller and its subcontrollers
///
pub struct ViewModelUpdateStream {
    /// The root controller whose updates we should return
    root_controller: Weak<dyn Controller>,

    /// Stream of updates from the root controller
    controller_stream: BoxStream<'static, Control>,

    /// Updates for the controller viewmodel
    controller_viewmodel_updates: Option<BoxStream<'static, ViewModelChange>>,

    /// The streams for the subcontrollers
    sub_controllers: HashMap<String, ViewModelUpdateStream>,

    /// The list of pending subcontroller updates
    pending: VecDeque<ViewModelUpdate>
}

impl ViewModelUpdateStream {
    ///
    /// Creates a new viewmodel update stream with the specified root controller
    ///
    pub fn new(root_controller: Arc<dyn Controller>) -> ViewModelUpdateStream {
        let ui                              = root_controller.ui();
        let controller_stream               = stream::iter(iter::once(ui.get())).chain(follow(ui));
        let controller_viewmodel_updates    = root_controller.get_viewmodel().map(|viewmodel| viewmodel.get_updates());
        let root_controller                 = Arc::downgrade(&root_controller);

        ViewModelUpdateStream {
            root_controller:                root_controller,
            controller_stream:              Box::pin(controller_stream),
            controller_viewmodel_updates:   controller_viewmodel_updates,
            sub_controllers:                HashMap::new(),
            pending:                        VecDeque::new()
        }
    }

    ///
    /// When the controller's UI changes, updates the subcontroller streams
    ///
    fn update_subcontrollers(&mut self, root_controller: &dyn Controller, control: &Control) {
        // Create a replacement set of subcontrollers
        let mut new_sub_controllers = HashMap::new();
        let all_controllers         = control.all_controllers();

        // For each subcontroller, either keep the existing stream or
        for subcontroller_name in all_controllers {
            if !new_sub_controllers.contains_key(&subcontroller_name) {
                if let Some(existing_controller) = self.sub_controllers.remove(&subcontroller_name) {
                    // Was already tracking this subcontroller
                    new_sub_controllers.insert(subcontroller_name, existing_controller);

                } else if let Some(subcontroller) = root_controller.get_subcontroller(&subcontroller_name) {
                    // Need to track with a new subcontroller
                    let subcontroller_stream = ViewModelUpdateStream::new(subcontroller);

                    new_sub_controllers.insert(subcontroller_name.clone(), subcontroller_stream);
                }
            }
        }

        // Replace the sub controllers with the new subcontrollers
        self.sub_controllers = new_sub_controllers;
    }
}

impl Stream for ViewModelUpdateStream {
    type Item = ViewModelUpdate;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<ViewModelUpdate>> {
        let self_ref = self.get_mut();

        if let Some(update) = self_ref.pending.pop_front() {
            // Return pending items before anything else
            Poll::Ready(Some(update))
        } else if let Some(root_controller) = self_ref.root_controller.upgrade() {
            // Try the updates from the main controller first
            if let Some(controller_viewmodel_updates) = self_ref.controller_viewmodel_updates.as_mut() {
                let mut all_updates = vec![];

                // Drain the controller updates
                let mut update_poll = controller_viewmodel_updates.poll_next_unpin(context);

                while let Poll::Ready(Some(update)) = update_poll {
                    match update {
                        ViewModelChange::NewProperty(name, value) => {
                            all_updates.push(ViewModelChange::NewProperty(name, value));
                        }

                        ViewModelChange::PropertyChanged(name, value) => {
                            all_updates.push(ViewModelChange::PropertyChanged(name, value));
                        }
                    }

                    // Poll for the next update
                    update_poll = controller_viewmodel_updates.poll_next_unpin(context);
                }

                // Unset the controller updates if we reach the end of the stream (the controller and its subcontrollers presumably still exist, so the stream does not end)
                if update_poll == Poll::Ready(None) {
                    // TODO: remove the viewmodel updates from self (borrowed, so doesn't work)
                    // self_ref.controller_viewmodel_updates = None;
                }

                // Return the updates if there were any
                if all_updates.len() > 0 {
                    self_ref.pending.push_back(ViewModelUpdate::new(vec![], all_updates));
                }
            }

            // Check for updates to the controller UI
            let mut next_ui_poll = self_ref.controller_stream.poll_next_unpin(context);
            while let Poll::Ready(Some(next_ui)) = next_ui_poll {
                // Refresh the subcontrollers from the UI
                self_ref.update_subcontrollers(&*root_controller, &next_ui);

                // Keep polling
                next_ui_poll = self_ref.controller_stream.poll_next_unpin(context);
            }

            if let Poll::Ready(None) = next_ui_poll {
                // If the controller's UI stream ends, then the viewmodel updates also end (presumably the controller has been disposed of)
                return Poll::Ready(None);
            }

            // Poll the subcontrollers
            let mut removed_subcontrollers = vec![];
            for (name, stream) in self_ref.sub_controllers.iter_mut() {
                let mut subcontroller_poll = stream.poll_next_unpin(context);

                while let Poll::Ready(Some(mut update)) = subcontroller_poll {
                    // Add the name of this subcontroller
                    update.add_to_start_of_path(name.clone());

                    // Add the update to the pending list
                    self_ref.pending.push_back(update);

                    // Fetch as many updates as we can from the subcontroller
                    subcontroller_poll = stream.poll_next_unpin(context);
                }

                if let Poll::Ready(None) = subcontroller_poll {
                    // This subcontroller has gone away and is no longer producing updates
                    removed_subcontrollers.push(name.clone());
                }
            }

            // Remove and try to recreate any subcontrollers that have stopped responding
            for removed_name in removed_subcontrollers {
                // Stop checking this subcontroller for updates
                self_ref.sub_controllers.remove(&removed_name);

                // Try to get it back from the root controller
                let new_subcontroller = root_controller.get_subcontroller(&removed_name);
                if let Some(new_subcontroller) = new_subcontroller {
                    // Got a replacement
                    let new_viewmodel_stream = ViewModelUpdateStream::new(new_subcontroller);
                    self_ref.sub_controllers.insert(removed_name, new_viewmodel_stream);

                    // Make sure the task is notified to re-poll for the changes from the replacement subcontroller
                    context.waker().clone().wake();
                }
            }

            // If any updates were found, return the first from the pending list
            if let Some(update) = self_ref.pending.pop_front() {
                Poll::Ready(Some(update))
            } else {
                Poll::Pending
            }
        } else {
            // Stream has ended when the root controller no longer exists
            Poll::Ready(None)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use super::super::super::property::*;
    use super::super::super::dynamic_viewmodel::*;

    use futures::executor;

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

    #[derive(Clone)]
    struct NotifyNothing;
    impl task::ArcWake for NotifyNothing {
        fn wake_by_ref(_arc_self: &Arc<Self>) { }
    }

    #[test]
    fn changes_are_picked_up() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let mut stream  = ViewModelUpdateStream::new(controller.clone());

        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        executor::block_on(async {
            assert!(stream.next().await == Some(ViewModelUpdate::new(vec![], vec![ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(2))])));
        });
    }

    #[test]
    fn changes_are_picked_up_as_changes() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let mut stream  = ViewModelUpdateStream::new(controller.clone());

        executor::block_on(async {
            assert!(stream.next().await == Some(ViewModelUpdate::new(vec![], vec![ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(1))])));

            controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

            assert!(stream.next().await == Some(ViewModelUpdate::new(vec![], vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))])));
        });
    }

    #[test]
    fn new_values_are_picked_up() {
        let controller  = Arc::new(DynamicController::new());
        let mut stream  = ViewModelUpdateStream::new(controller.clone());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        executor::block_on(async {
            assert!(stream.next().await == Some(ViewModelUpdate::new(vec![], vec![ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(1))])));

            controller.get_viewmodel().unwrap().set_property("NewValue", PropertyValue::Int(2));

            assert!(stream.next().await == Some(ViewModelUpdate::new(vec![], vec![ViewModelChange::NewProperty("NewValue".to_string(), PropertyValue::Int(2))])));
        })
    }

    #[test]
    fn new_values_are_picked_up_alongside_changes() {
        let controller = Arc::new(DynamicController::new());
        controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        executor::block_on(async {
            let mut stream  = ViewModelUpdateStream::new(controller.clone());
            assert!(stream.next().await == Some(ViewModelUpdate::new(vec![], vec![ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(1))])));

            controller.get_viewmodel().unwrap().set_property("NewValue", PropertyValue::Int(3));
            controller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

            let events = stream.next().await;
            println!("{:?}", events);
            assert!(events == Some(ViewModelUpdate::new(vec![], vec![ViewModelChange::NewProperty("NewValue".to_string(), PropertyValue::Int(3)), ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))])));
        });
    }

    #[test]
    fn subcontroller_changes_are_picked_up() {
        let controller = DynamicController::new();
        controller.set_controls(Control::container().with_controller("Subcontroller"));
        controller.add_subcontroller("Subcontroller".to_string());

        let subcontroller = controller.get_subcontroller("Subcontroller").unwrap();
        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(1));

        let controller = Arc::new(controller);

        let update_stream       = ViewModelUpdateStream::new(controller.clone());
        let mut update_stream   = update_stream;

        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        executor::block_on(async {
            let update = update_stream.next().await.unwrap();

            assert!(update.controller_path() == &vec!["Subcontroller".to_string()]);
            assert!(update.updates() == &vec![ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(2))]);
        });
    }

    #[test]
    fn new_controller_is_picked_up() {
        let controller = DynamicController::new();
        controller.set_controls(Control::container());

        let controller = Arc::new(controller);

        let update_stream       = ViewModelUpdateStream::new(controller.clone());
        let mut update_stream   = update_stream;

        controller.set_controls(Control::container().with_controller("Subcontroller"));
        controller.add_subcontroller("Subcontroller".to_string());
        let subcontroller = controller.get_subcontroller("Subcontroller").unwrap();

        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        executor::block_on(async {
            let updates = update_stream.next().await.unwrap();

            assert!(updates.controller_path() == &vec!["Subcontroller".to_string()]);
            assert!(updates.updates() == &vec![ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(2))]);
        });
    }

    #[test]
    fn changes_after_new_controller_are_picked_up() {
        let controller = DynamicController::new();
        controller.set_controls(Control::container());

        let controller = Arc::new(controller);

        let update_stream       = ViewModelUpdateStream::new(controller.clone());
        let mut update_stream   = update_stream;

        controller.set_controls(Control::container().with_controller("Subcontroller"));
        controller.add_subcontroller("Subcontroller".to_string());
        let subcontroller = controller.get_subcontroller("Subcontroller").unwrap();

        subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(2));

        executor::block_on(async {
            let _updates = update_stream.next().await.unwrap();

            subcontroller.get_viewmodel().unwrap().set_property("Test", PropertyValue::Int(3));
            let updates = update_stream.next().await.unwrap();

            assert!(updates.controller_path() == &vec!["Subcontroller".to_string()]);
            assert!(updates.updates() == &vec![ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(3))]);
        });
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

        fn get_updates(&self) -> BoxStream<'static, ViewModelChange> {
            Box::pin(stream::iter(vec![
                ViewModelChange::NewProperty("Test1".to_string(), PropertyValue::String("Test1".to_string())),
                ViewModelChange::NewProperty("Test2".to_string(), PropertyValue::String("Test2".to_string())),
                ViewModelChange::NewProperty("Test3".to_string(), PropertyValue::String("Test3".to_string()))
            ]).chain(stream::poll_fn(|_context| Poll::Pending)))
        }
    }

    #[test]
    pub fn generate_initial_controller_events() {
        executor::block_on(async {
            for _pass in 0..10 {
                let controller          = Arc::new(TestController::new());
                let update_stream       = ViewModelUpdateStream::new(controller.clone());
                let mut update_stream   = update_stream;

                let update1 = update_stream.next().await;
                println!("{:?}", update1);
                let update2 = update_stream.next().await;
                println!("{:?}", update2);

                let update1 = update1.unwrap();
                let update2 = update2.unwrap();

                let (update1, update2) = if update2.controller_path() == &vec!["Model1".to_string()] {
                    (update2, update1)
                } else {
                    (update1, update2)
                };

                assert!(update1.controller_path() == &vec!["Model1".to_string()]);
                assert!(update1.updates() == &vec![
                    ViewModelChange::NewProperty("Test1".to_string(), PropertyValue::String("Test1".to_string())),
                    ViewModelChange::NewProperty("Test2".to_string(), PropertyValue::String("Test2".to_string())),
                    ViewModelChange::NewProperty("Test3".to_string(), PropertyValue::String("Test3".to_string())),
                ]);

                assert!(update2.controller_path() == &vec!["Model2".to_string()]);
                assert!(update2.updates() == &vec![
                    ViewModelChange::NewProperty("Test1".to_string(), PropertyValue::String("Test1".to_string())),
                    ViewModelChange::NewProperty("Test2".to_string(), PropertyValue::String("Test2".to_string())),
                    ViewModelChange::NewProperty("Test3".to_string(), PropertyValue::String("Test3".to_string())),
                ]);
            }
        });
    }

    // TODO: detects removed controller
}

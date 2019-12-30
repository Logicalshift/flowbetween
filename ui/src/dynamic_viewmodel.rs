use super::property::*;
use super::viewmodel::*;

use desync::{Desync};
use flo_stream::*;

use flo_binding::*;
use flo_binding::binding_context::*;

use futures::*;
use futures::stream;
use futures::stream::{BoxStream};
use futures::task;
use futures::task::{Poll, Context, Waker};
use std::pin::*;
use std::sync::*;
use std::collections::{HashMap, VecDeque};

///
/// The dynamic viewmodel lets us define arbitrary properties as bound or
/// computed values. A particular key can only be bound or computed: if it
/// is set as both, the computed version 'wins'.
///
pub struct DynamicViewModel {
    /// Stream of new properties being created for this viewmodel
    new_properties: Desync<Publisher<(String, BindRef<PropertyValue>)>>,

    /// Maps bindings in this viewmodel to their values
    bindings: Mutex<HashMap<String, Arc<Binding<PropertyValue>>>>,

    /// Maps computed bindings to their values (we ignore these when setting)
    computed: Mutex<HashMap<String, BindRef<PropertyValue>>>,

    /// Used for properties that don't exist in this model
    nothing: BindRef<PropertyValue>
}

///
/// Notifier for a dynamic stream
///
struct DynamicStreamNotify {
    /// Task to notify
    waker: Mutex<Option<Waker>>,

    /// True if this stream has notified
    was_notified: Mutex<bool>
}

///
/// Single property being streamed
///
struct DynamicStreamProperty {
    /// Last value returned to this stream
    last_value: Option<PropertyValue>,

    /// Stream of values for this item
    value_stream: BoxStream<'static, PropertyValue>,

    /// Most recent notifier for this stream
    notify: Arc<DynamicStreamNotify>
}

///
/// Stream implementation that polls the forwarded changes futures when it's polled
///
/// We could also pipe changes into desync, but this has the advantage that it will actually
/// 'pull' changes in on the current thread rather than generate them asynchronously on a
/// different thread, which is useful when trying to drain all updates from the publisher.
///
struct DynamicViewModelUpdateStream<NewProperties: Stream<Item=(String, BindRef<PropertyValue>)>+Unpin> {
    /// Stream of new property bindings
    new_properties: NewProperties,

    // Newly created properties waiting to be returned by the stream
    pending_changes: VecDeque<ViewModelChange>,

    /// Stream that monitors for any property change in the viewmodel
    any_property: HashMap<String, DynamicStreamProperty>
}

impl task::ArcWake for DynamicStreamNotify {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Set the flag
        *arc_self.was_notified.lock().unwrap() = true;

        // Wake the task
        arc_self.waker.lock().unwrap()
            .take()
            .map(|waker| waker.wake());
    }
}

impl<NewProperties: Stream<Item=(String, BindRef<PropertyValue>)>+Unpin> Stream for DynamicViewModelUpdateStream<NewProperties> {
    type Item = ViewModelChange;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<ViewModelChange>> {
        // Set up any new properties
        let mut new_property_poll = self.new_properties.poll_next_unpin(context);
        while let Poll::Ready(Some((name, binding))) = new_property_poll {
            // Create a new property with its notify flag set
            let notified = DynamicStreamNotify {
                waker:          Mutex::new(Some(context.waker().clone())),
                was_notified:   Mutex::new(true)
            };

            let value = binding.get();

            let property = DynamicStreamProperty {
                last_value:     Some(value.clone()),
                value_stream:   Box::pin(follow(binding)),
                notify:         Arc::new(notified)
            };

            // Push to the pending changes list
            self.pending_changes.push_back(ViewModelChange::NewProperty(name.clone(), value));

            // Add to the list of properties this stream is following
            self.any_property.insert(name, property);

            // Get the next new property
            new_property_poll = self.new_properties.poll_next_unpin(context);
        }

        // Return pending changes first
        if let Some(next_change) = self.pending_changes.pop_front() {
            return Poll::Ready(Some(next_change));
        }

        // If the new properties stream comes to an end, this stream has come to an end
        if let Poll::Ready(None) = new_property_poll {
            return Poll::Ready(None);
        }

        // Poll for values from any properties with their flag set
        for (name, property) in self.any_property.iter_mut() {
            // Update the task to notify
            *property.notify.waker.lock().unwrap() = Some(context.waker().clone());

            // If the flag is set...
            if *property.notify.was_notified.lock().unwrap() {
                // Try polling this item
                loop {
                    let notify_waker        = task::waker(Arc::clone(&property.notify));
                    let mut notify_context  = Context::from_waker(&notify_waker);
                    let notify_poll         = property.value_stream.poll_next_unpin(&mut notify_context);

                    match notify_poll {
                        Poll::Ready(Some(new_value)) => {
                            // Got an update for this property
                            if Some(&new_value) != property.last_value.as_ref() {
                                // Store the last value so we don't create duplicate updates
                                property.last_value = Some(new_value.clone());

                                // Value changed: send it on
                                let update = ViewModelChange::PropertyChanged(name.clone(), new_value);
                                return Poll::Ready(Some(update));
                            } else {
                                // Poll again in case there is an actual new value (or to start waiting for updates on this property)
                            }
                        },

                        Poll::Ready(None) => {
                            // Property was deleted
                            *property.notify.was_notified.lock().unwrap() = false;

                            // Need to keep polling this item
                            break;
                        },

                        Poll::Pending => {
                            // Not notified any more
                            *property.notify.was_notified.lock().unwrap() = false;

                            // Need to keep polling this item
                            break;
                        }
                    }
                }
            }
        }

        // No updates available
        Poll::Pending
    }
}

impl DynamicViewModel {
    ///
    /// Creates a new dynamic viewmodel
    ///
    pub fn new() -> DynamicViewModel {
        DynamicViewModel {
            new_properties:     Desync::new(Publisher::new(100)),
            bindings:           Mutex::new(HashMap::new()),
            computed:           Mutex::new(HashMap::new()),
            nothing:            BindRef::from(bind(PropertyValue::Nothing)) }
    }

    ///
    /// Attempts to retrieve the set binding with a particular name
    ///
    fn get_binding(&self, property_name: &str) -> Option<Arc<Binding<PropertyValue>>> {
        let bindings = self.bindings.lock().unwrap();

        bindings.get(&String::from(property_name)).map(|arc| arc.clone())
    }

    ///
    /// Attempts to retrieve the computed binding with a paritcular name
    ///
    fn get_computed(&self, property_name: &str) -> Option<BindRef<PropertyValue>> {
        let computed = self.computed.lock().unwrap();

        computed.get(&String::from(property_name)).map(|arc| arc.clone())
    }

    ///
    /// Sets a binding to a computed value
    ///
    pub fn set_computed<TFn>(&self, property_name: &str, calculate_value: TFn)
    where TFn: 'static+Send+Sync+Fn() -> PropertyValue {
        // If this is done while computing the UI, we don't want our computed to attach to the current context
        BindingContext::out_of_context(move || {
            let new_binding = BindRef::from(computed(calculate_value));

            let mut computed = self.computed.lock().unwrap();
            computed.insert(String::from(property_name), new_binding.clone());

            self.follow_binding(property_name, new_binding);
        });
    }

    ///
    /// Returns true if the specified binding exists in this viewmodel
    ///
    pub fn has_binding(&self, property_name: &str) -> bool {
        if self.bindings.lock().unwrap().contains_key(property_name) {
            true
        } else if self.computed.lock().unwrap().contains_key(property_name) {
            true
        } else {
            false
        }
    }

    ///
    /// Follows a binding and publishes updates to the update stream
    ///
    fn follow_binding<TBinding: 'static+Bound<PropertyValue>>(&self, property_name: &str, binding: TBinding) {
        // Use a future to publish the property
        let property_name   = String::from(property_name);
        let _future         = self.new_properties.future(move |new_properties| {
            new_properties.publish((String::from(property_name), BindRef::from_arc(Arc::new(binding))))
        });

        // Synchronise with where the future completes, so the property is ready by the time this call returns
        self.new_properties.sync(|_| { });
    }
}

impl ViewModel for DynamicViewModel {
    fn get_property(&self, property_name: &str) -> BindRef<PropertyValue> {
        if let Some(result) = self.get_computed(property_name) {
            // Computed values are returned first, so these bindings cannot be set
            result
        } else if let Some(result) = self.get_binding(property_name) {
            // 'Set' bindings are returned next
            BindRef::from_arc(result)
        } else {
            // If an invalid name is requested, we return something bound to nothing
            self.nothing.clone()
        }
    }

    fn set_property(&self, property_name: &str, new_value: PropertyValue) {
        let mut bindings = self.bindings.lock().unwrap();

        if let Some(value) = bindings.get(&String::from(property_name)) {
            // Update the binding
            (**value).set(new_value);

            // Awkward return because rust keeps the borrow in the else clause even though nothing can reference it
            return;
        }

        // Property does not exist in this viewmodel: create a new one
        let new_binding = bind(new_value);
        bindings.insert(String::from(property_name), Arc::new(new_binding.clone()));
        self.follow_binding(property_name, new_binding);
    }

    fn get_property_names(&self) -> Vec<String> {
        // The keys for items with 'set' bindings
        let mut binding_keys: Vec<_> = {
            let bindings = self.bindings.lock().unwrap();
            bindings
                .keys()
                .map(|key| key.clone())
                .collect()
        };

        // Keys for items with computed bindings
        let mut computed_keys: Vec<_> = {
            let computed = self.computed.lock().unwrap();
            computed
                .keys()
                .map(|key| key.clone())
                .collect()
        };

        // Combine them and deduplicate for the final list of keys
        binding_keys.append(&mut computed_keys);
        binding_keys.sort();
        binding_keys.dedup();

        binding_keys
    }

    fn get_updates(&self) -> BoxStream<'static, ViewModelChange> {
        // Gather the existing bindings
        let existing_properties = self.bindings.lock().unwrap().iter()
            .map(|(name, binding)| (name.clone(), BindRef::from_arc(Arc::clone(binding))))
            .collect::<Vec<_>>();
        let existing_computed   = self.computed.lock().unwrap().iter()
            .map(|(name, binding)| (name.clone(), binding.clone()))
            .collect::<Vec<_>>();

        // Subscribe to any new properties that might be added after the stream is generated
        let new_properties      = self.new_properties.sync(|new_properties| new_properties.subscribe());

        // Initially all properties are new
        let existing_properties = stream::iter(existing_properties);
        let existing_computed   = stream::iter(existing_computed);
        let new_properties      = existing_properties.chain(existing_computed).chain(new_properties);

        // Create the new stream
        let stream = DynamicViewModelUpdateStream {
            new_properties:     new_properties,
            pending_changes:    VecDeque::new(),
            any_property:       HashMap::new()
        };

        Box::pin(stream)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nonexistent_value_is_nothing() {
        let viewmodel = DynamicViewModel::new();

        assert!(viewmodel.get_property("Test").get() == PropertyValue::Nothing);
    }

    #[test]
    fn can_set_value() {
        let viewmodel = DynamicViewModel::new();

        viewmodel.set_property("Test", PropertyValue::Int(2));

        assert!(viewmodel.get_property("Test").get() == PropertyValue::Int(2));
    }

    #[test]
    fn can_compute_value() {
        let viewmodel = DynamicViewModel::new();

        viewmodel.set_computed("Test", || PropertyValue::Int(2));

        assert!(viewmodel.get_property("Test").get() == PropertyValue::Int(2));
    }

    #[test]
    fn computed_value_updates() {
        let viewmodel = DynamicViewModel::new();

        viewmodel.set_property("TestSource", PropertyValue::Int(1));

        let test_source = viewmodel.get_property("TestSource");
        viewmodel.set_computed("Test", move || test_source.get());

        assert!(viewmodel.get_property("Test").get() == PropertyValue::Int(1));

        viewmodel.set_property("TestSource", PropertyValue::Int(2));

        assert!(viewmodel.get_property("Test").get() == PropertyValue::Int(2));
    }

    #[test]
    fn stream_returns_updates() {
        let viewmodel = DynamicViewModel::new();
        viewmodel.set_property("Test", PropertyValue::Int(2));

        let mut updates = viewmodel.get_updates();

        executor::block_on(async {
            viewmodel.set_property("Test", PropertyValue::Int(3));
            assert!(updates.next().await == Some(ViewModelChange::NewProperty(String::from("Test"), PropertyValue::Int(3))));

            viewmodel.set_property("Test", PropertyValue::Int(4));
            assert!(updates.next().await == Some(ViewModelChange::PropertyChanged(String::from("Test"), PropertyValue::Int(4))));
        });
    }

    #[test]
    fn stream_skips_updates() {
        let viewmodel = DynamicViewModel::new();
        viewmodel.set_property("Test", PropertyValue::Int(2));

        let mut updates = viewmodel.get_updates();
        viewmodel.set_property("Test", PropertyValue::Int(3));
        viewmodel.set_property("Test", PropertyValue::Int(4));
        viewmodel.set_property("Test", PropertyValue::Int(5));

        executor::block_on(async {
            assert!(updates.next().await == Some(ViewModelChange::NewProperty(String::from("Test"), PropertyValue::Int(5))));
        });
    }

    #[test]
    fn new_values_are_picked_up() {
        let viewmodel   = DynamicViewModel::new();
        let mut stream  = viewmodel.get_updates();
        viewmodel.set_property("Test", PropertyValue::Int(1));

        executor::block_on(async {
            assert!(stream.next().await == Some(ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(1))));

            viewmodel.set_property("NewValue", PropertyValue::Int(2));

            assert!(stream.next().await == Some(ViewModelChange::NewProperty("NewValue".to_string(), PropertyValue::Int(2))));
        })
    }

    #[test]
    fn changes_are_picked_up() {
        let viewmodel   = DynamicViewModel::new();
        let mut stream  = viewmodel.get_updates();
        viewmodel.set_property("Test", PropertyValue::Int(1));

        executor::block_on(async {
            assert!(stream.next().await == Some(ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(1))));

            viewmodel.set_property("Test", PropertyValue::Int(2));

            assert!(stream.next().await == Some(ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))));
        })
    }

    #[test]
    fn notifications_are_posted() {
        use std::thread;
        use std::time::{Duration};

        let viewmodel   = DynamicViewModel::new();
        let mut stream  = viewmodel.get_updates();
        viewmodel.set_property("Test", PropertyValue::Int(1));

        executor::block_on(async {
            assert!(stream.next().await == Some(ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(1))));

            thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                viewmodel.set_property("Test", PropertyValue::Int(2));

                // Timeout before destroying the viewmodel (will end the stream, failing the test)
                thread::sleep(Duration::from_millis(100));
            });

            let event = stream.next().await;
            println!("{:?}", event);
            assert!(event == Some(ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))));
        })
    }

    #[test]
    fn new_values_are_picked_up_alongside_changes() {
        // Equivalent test is in the viewmodel_stream
        let viewmodel = DynamicViewModel::new();
        viewmodel.set_property("Test", PropertyValue::Int(1));

        executor::block_on(async {
            let mut stream  = viewmodel.get_updates();

            let events = stream.next().await;
            println!("{:?}", events);
            assert!(events == Some(ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(1))));

            viewmodel.set_property("NewValue", PropertyValue::Int(3));
            viewmodel.set_property("Test", PropertyValue::Int(2));

            let events = stream.next().await;
            println!("{:?}", events);
            assert!(events == Some(ViewModelChange::NewProperty("NewValue".to_string(), PropertyValue::Int(3))));

            let events = stream.next().await;
            println!("{:?}", events);
            assert!(events == Some(ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))));
        });
    }

    #[test]
    fn new_values_are_picked_up_alongside_changes_no_buffering() {
        // Equivalent test is in the viewmodel_stream
        let viewmodel = DynamicViewModel::new();
        viewmodel.set_property("Test", PropertyValue::Int(1));

        executor::block_on(async {
            let mut stream  = viewmodel.get_updates();

            let events = stream.next().await;
            println!("{:?}", events);
            assert!(events == Some(ViewModelChange::NewProperty("Test".to_string(), PropertyValue::Int(1))));

            viewmodel.set_property("NewValue", PropertyValue::Int(3));

            let events = stream.next().await;
            println!("{:?}", events);
            assert!(events == Some(ViewModelChange::NewProperty("NewValue".to_string(), PropertyValue::Int(3))));

            viewmodel.set_property("Test", PropertyValue::Int(2));

            let events = stream.next().await;
            println!("{:?}", events);
            assert!(events == Some(ViewModelChange::PropertyChanged("Test".to_string(), PropertyValue::Int(2))));
        });
    }

    #[test]
    fn stream_indicates_new_properties() {
        let viewmodel = DynamicViewModel::new();
        viewmodel.set_property("Test", PropertyValue::Int(2));

        let mut updates = viewmodel.get_updates();

        executor::block_on(async {
            viewmodel.set_property("Test", PropertyValue::Int(3));
            assert!(updates.next().await == Some(ViewModelChange::NewProperty(String::from("Test"), PropertyValue::Int(3))));

            viewmodel.set_property("Test2", PropertyValue::Int(4));
            assert!(updates.next().await == Some(ViewModelChange::NewProperty(String::from("Test2"), PropertyValue::Int(4))));

            viewmodel.set_property("Test2", PropertyValue::Int(5));
            assert!(updates.next().await == Some(ViewModelChange::PropertyChanged(String::from("Test2"), PropertyValue::Int(5))));
        });
    }

    #[test]
    fn stream_computed_values() {
        let viewmodel = DynamicViewModel::new();
        let mut updates = viewmodel.get_updates();

        executor::block_on(async {
            viewmodel.set_property("TestSource", PropertyValue::Int(1));
            assert!(updates.next().await == Some(ViewModelChange::NewProperty(String::from("TestSource"), PropertyValue::Int(1))));

            let test_source = viewmodel.get_property("TestSource");
            viewmodel.set_computed("Test", move || test_source.get());

            assert!(viewmodel.get_property("Test").get() == PropertyValue::Int(1));
            assert!(updates.next().await == Some(ViewModelChange::NewProperty(String::from("Test"), PropertyValue::Int(1))));

            viewmodel.set_property("TestSource", PropertyValue::Int(2));
            let update1 = updates.next().await;
            let update2 = updates.next().await;

            // Order of updates is indeterminate
            println!("{:?}", update1);
            println!("{:?}", update2);

            if update1 == Some(ViewModelChange::PropertyChanged(String::from("TestSource"), PropertyValue::Int(2))) {
                assert!(update2 == Some(ViewModelChange::PropertyChanged(String::from("Test"), PropertyValue::Int(2))));

            } else if update1 == Some(ViewModelChange::PropertyChanged(String::from("Test"), PropertyValue::Int(2))) {
                assert!(update2 == Some(ViewModelChange::PropertyChanged(String::from("TestSource"), PropertyValue::Int(2))));

            } else {
                assert!(false);
            }
        });
    }

    #[test]
    fn property_value_notifies_without_viewmodel() {
        let notified    = Arc::new(Mutex::new(false));

        // For the viewmodel to work, we need property value changes to trigger a notification
        let property_value          = bind(PropertyValue::Int(1));

        let computed_source_value   = property_value.clone();
        let computed_property       = computed(move || computed_source_value.get());

        let test_value_notified = notified.clone();
        computed_property.when_changed(notify(move || (*test_value_notified.lock().unwrap()) = true)).keep_alive();

        assert!(computed_property.get() == PropertyValue::Int(1));
        assert!((*notified.lock().unwrap()) == false);

        property_value.set(PropertyValue::Int(2));

        assert!(computed_property.get() == PropertyValue::Int(2));
        assert!((*notified.lock().unwrap()) == true);
    }

    #[test]
    fn standard_value_notifies_after_propagation() {
        let notified    = Arc::new(Mutex::new(false));
        let viewmodel   = DynamicViewModel::new();

        // Creates the 'TestSource' property
        viewmodel.set_property("TestSource", PropertyValue::Int(1));

        // Computes a value equal to the current TestSource property
        let test_source = viewmodel.get_property("TestSource");
        let test_value  = computed(move || test_source.get());

        // Whenever it changes, set a flag
        let test_value_notified = notified.clone();
        test_value.when_changed(notify(move || (*test_value_notified.lock().unwrap()) = true)).keep_alive();

        // Initially unchanged
        assert!(test_value.get() == PropertyValue::Int(1));
        assert!((*notified.lock().unwrap()) == false);

        // Updating the value should cause the notification to fiew
        viewmodel.set_property("TestSource", PropertyValue::Int(2));

        assert!(viewmodel.get_property("TestSource").get() == PropertyValue::Int(2));
        assert!(test_value.get() == PropertyValue::Int(2));
        assert!((*notified.lock().unwrap()) == true);
    }

    #[test]
    fn computed_value_notifies() {
        let notified    = Arc::new(Mutex::new(false));
        let viewmodel   = DynamicViewModel::new();

        viewmodel.set_property("TestSource", PropertyValue::Int(1));

        let test_source = viewmodel.get_property("TestSource");
        viewmodel.set_computed("Test", move || test_source.get());

        let test_value_notified = notified.clone();
        viewmodel.get_property("Test").when_changed(notify(move || (*test_value_notified.lock().unwrap()) = true)).keep_alive();

        assert!(viewmodel.get_property("Test").get() == PropertyValue::Int(1));
        assert!((*notified.lock().unwrap()) == false);

        viewmodel.set_property("TestSource", PropertyValue::Int(2));

        assert!(viewmodel.get_property("Test").get() == PropertyValue::Int(2));
        assert!((*notified.lock().unwrap()) == true);
    }

    #[test]
    fn computed_value_notifies_after_propagation() {
        let notified    = Arc::new(Mutex::new(false));
        let viewmodel   = DynamicViewModel::new();

        viewmodel.set_property("TestSource", PropertyValue::Int(1));

        let test_source = viewmodel.get_property("TestSource");
        viewmodel.set_computed("Test", move || test_source.get());

        let test        = viewmodel.get_property("Test");
        let test_value  = computed(move || test.get());

        let test_value_notified = notified.clone();
        test_value.when_changed(notify(move || (*test_value_notified.lock().unwrap()) = true)).keep_alive();

        assert!(test_value.get() == PropertyValue::Int(1));
        assert!((*notified.lock().unwrap()) == false);

        viewmodel.set_property("TestSource", PropertyValue::Int(2));

        assert!(viewmodel.get_property("Test").get() == PropertyValue::Int(2));
        assert!(test_value.get() == PropertyValue::Int(2));
        assert!((*notified.lock().unwrap()) == true);
    }
}

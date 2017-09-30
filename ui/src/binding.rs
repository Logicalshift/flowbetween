//!
//! # Bindings
//!
//! This provides a means for building data-driven applications. The
//! basic model is similar to how spreadsheets work: we watch what
//! items a particular calculation depends on and generate an event
//! when any of these change.
//!

use std::mem;
use std::sync::*;
use std::rc::*;
use std::cell::*;

///
/// Trait implemented by items with dependencies that need to be notified when they have changed
///
pub trait Notifiable : Sync+Send {
    ///
    /// Indicates that a dependency of this object has changed
    ///
    fn mark_as_changed(&self);
}

///
/// Trait implemented by an object that can be released
///
pub trait Releasable : Send {
    ///
    /// Indicates that this object is finished with and should be released
    ///
    fn done(&mut self);
}

///
/// Trait implemented by items that can notify something when they're changed
///
pub trait Changeable {
    ///
    /// Supplies an item to be notified when this item is changed
    ///
    fn when_changed(&mut self, what: Arc<Notifiable>) -> Box<Releasable>;
}

///
/// Trait implemented by something that is bound to a value
///
pub trait Bound<Value> : Changeable {
    ///
    /// Retrieves the value stored by this binding
    ///
    fn get(&self) -> Value;
}

///
/// Trait implemented by something that is bound to a value that can be changed
///
pub trait MutableBound<Value> : Bound<Value> {
    ///
    /// Sets the value stored by this binding
    ///
    fn set(&mut self, new_value: Value);
}

///
/// A notifiable that can be released (and then tidied up later)
///
#[derive(Clone)]
struct ReleasableNotifiable {
    target: Arc<Mutex<RefCell<Option<Arc<Notifiable>>>>>
}

impl ReleasableNotifiable {
    ///
    /// Creates a new releasable notifiable object
    ///
    fn new(target: Arc<Notifiable>) -> ReleasableNotifiable {
        ReleasableNotifiable {
            target: Arc::new(Mutex::new(RefCell::new(Some(target))))
        }
    }

    ///
    /// Marks this as changed and returns whether or not the notification was called
    ///
    fn mark_as_changed(&self) -> bool {
        // Reset the optional item so that it's 'None'
        let lock = self.target.lock().unwrap();

        // Send to the target
        if lock.borrow().is_some() {
            lock.borrow().as_ref().unwrap().mark_as_changed();
            true
        } else {
            false
        }
    }

    ///
    /// True if this item is still in use
    ///
    fn is_in_use(&self) -> bool {
        let lock    = self.target.lock().unwrap();
        let result  = lock.borrow().is_some();
        result
    }
}

impl Releasable for ReleasableNotifiable {
    fn done(&mut self) {
        // Reset the optional item so that it's 'None'
        let lock = self.target.lock().unwrap();

        *lock.borrow_mut() = None;
    }
}

impl Releasable for Vec<Box<Releasable>> {
    fn done(&mut self) {
        for item in self.iter_mut() {
            item.done();
        }
    }
}

impl Notifiable for ReleasableNotifiable {
    fn mark_as_changed(&self) {
        // Reset the optional item so that it's 'None'
        let lock = self.target.lock().unwrap();

        // Send to the target
        lock.borrow().as_ref().map(|target| target.mark_as_changed());
    }
}

///
/// Represents the dependencies of a binding context
///
#[derive(Clone)]
pub struct BindingDependencies {
    /// The list of changables that are dependent on this context
    dependencies: Rc<RefCell<Vec<Box<Changeable>>>>
}

impl BindingDependencies {
    ///
    /// Creates a new binding dependencies object
    ///
    pub fn new() -> BindingDependencies {
        BindingDependencies { dependencies: Rc::new(RefCell::new(vec![])) }
    }

    ///
    /// Adds a new dependency to this object
    ///
    pub fn add_dependency<TChangeable: Changeable+'static>(&mut self, dependency: TChangeable) {
        self.dependencies.borrow_mut().push(Box::new(dependency))
    }
}

impl Changeable for BindingDependencies {
    fn when_changed(&mut self, what: Arc<Notifiable>) -> Box<Releasable> {
        let mut to_release = vec![];

        for dep in self.dependencies.borrow_mut().iter_mut() {
            to_release.push(dep.when_changed(what.clone()));
        }

        Box::new(to_release)
    }
}

///
/// Represents a binding context. Binding contexts are
/// per-thread structures, used to track 
///
#[derive(Clone)]
pub struct BindingContext {
    /// The dependencies for this context
    dependencies: BindingDependencies,

    /// None, or the binding context that this context was created within
    nested: Option<Box<BindingContext>>
}

thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<BindingContext>> = RefCell::new(None);
}

impl BindingContext {
    ///
    /// Gets the active binding context
    ///
    pub fn current() -> Option<BindingContext> {
        CURRENT_CONTEXT.with(|current_context| {
            current_context
                .borrow()
                .as_ref()
                .map(|rc| rc.clone())
        })
    }

    ///
    /// Executes a function in a new binding context
    ///
    pub fn bind<TResult, TFn>(to_do: TFn) -> (TResult, BindingDependencies) 
    where TFn: FnOnce() -> TResult {
        // Remember the previous context
        let previous_context = Self::current();

        // Create a new context
        let dependencies    = BindingDependencies::new();
        let new_context     = BindingContext {
            dependencies:   dependencies.clone(),
            nested:         previous_context.clone().map(|ctx| Box::new(ctx))
        };

        // Make the current context the same as the new context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = Some(new_context));

        // Perform the requested action with this context
        let result = to_do();

        // Reset to the previous context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = previous_context);

        (result, dependencies)
    }

    pub fn add_dependency<TChangeable: Changeable+'static>(dependency: TChangeable) {
        Self::current().map(|mut ctx| ctx.dependencies.add_dependency(dependency));
    }
}

struct NotifyFn<TFn> {
    when_changed: Mutex<RefCell<TFn>>
}

impl<TFn> Notifiable for NotifyFn<TFn>
where TFn: Send+FnMut() -> () {
    fn mark_as_changed(&self) {
        let cell            = self.when_changed.lock().unwrap();
        let mut on_changed  = &mut *cell.borrow_mut();
        
        on_changed()
    }
}

///
/// An internal representation of a bound value
///
struct BoundValue<Value> {
    /// The current value of this binding
    value: Value,

    /// What to call when the value changes
    when_changed: Vec<ReleasableNotifiable>
}

impl<Value: Clone+PartialEq> BoundValue<Value> {
    ///
    /// Creates a new binding with the specified value
    ///
    pub fn new(val: Value) -> BoundValue<Value> {
        BoundValue {
            value:          val,
            when_changed:   vec![]
        }
    }

    ///
    /// Updates the value in this structure without calling the notifications, returns whether or not anything actually changed
    ///
    pub fn set_without_notifying(&mut self, new_value: Value) -> bool {
        let changed = self.value != new_value;

        self.value = new_value;

        changed
    }

    ///
    /// Retrieves a copy of the list of notifiable items for this value
    ///
    pub fn get_notifiable_items(&self) -> Vec<ReleasableNotifiable> {
        self.when_changed.clone()
    }

    ///
    /// If there are any notifiables in this object that aren't in use, remove them
    ///
    pub fn filter_unused_notifications(&mut self) {
        self.when_changed.retain(|releasable| releasable.is_in_use());
    }
}

impl<Value> Changeable for BoundValue<Value> {
    fn when_changed(&mut self, what: Arc<Notifiable>) -> Box<Releasable> {
        let releasable = ReleasableNotifiable::new(what);
        self.when_changed.push(releasable.clone());

        Box::new(releasable)
    }
}

impl<Value: Clone> Bound<Value> for BoundValue<Value> {
    fn get(&self) -> Value {
        self.value.clone()
    }
}

impl<Value: Clone+PartialEq> MutableBound<Value> for BoundValue<Value> {
    fn set(&mut self, new_value: Value) {
        if self.set_without_notifying(new_value) {
            let mut needs_filtering = false;

            for notify in self.when_changed.iter() {
                needs_filtering = needs_filtering || !notify.mark_as_changed();
            }

            if needs_filtering {
                self.filter_unused_notifications();
            }
        }
    }
}

///
/// Represents a thread-safe, sharable binding
///
#[derive(Clone)]
pub struct Binding<Value> {
    /// The value stored in this binding
    value: Arc<Mutex<RefCell<BoundValue<Value>>>>
}

impl<Value: Clone+PartialEq> Binding<Value> {
    fn new(value: Value) -> Binding<Value> {
        Binding {
            value: Arc::new(Mutex::new(RefCell::new(BoundValue::new(value))))
        }
    }
}

impl<Value> Changeable for Binding<Value> {
    fn when_changed(&mut self, what: Arc<Notifiable>) -> Box<Releasable> {
        let cell        = self.value.lock().unwrap();
        let releasable  = cell.borrow_mut().when_changed(what);

        releasable
    }
}

impl<Value: 'static+Clone> Bound<Value> for Binding<Value> {
    fn get(&self) -> Value {
        BindingContext::add_dependency(self.clone());

        let cell    = self.value.lock().unwrap();
        let value   = cell.borrow().get();

        value
    }
}

impl<Value: 'static+Clone+PartialEq> MutableBound<Value> for Binding<Value> {
    fn set(&mut self, new_value: Value) {
        // Update the value with the lock held
        let notifications = {
            let cell    = self.value.lock().unwrap();
            let changed = cell.borrow_mut().set_without_notifying(new_value);
        
            if changed {
                cell.borrow().get_notifiable_items()
            } else {
                vec![]
            }
        };

        // Call the notifications outside of the lock
        let mut needs_filtering = false;

        for to_notify in notifications.into_iter() {
            needs_filtering = needs_filtering || !to_notify.mark_as_changed();
        }

        if needs_filtering {
            let cell    = self.value.lock().unwrap();
            cell.borrow_mut().filter_unused_notifications();
        }
    }
}

///
/// Core representation ofa computed binding
///
struct ComputedBindingCore<Value: 'static+Clone+PartialEq, TFn>
where TFn: 'static+Fn() -> Value {
    /// Function to call to recalculate this item
    calculate_value: TFn,

    /// Most recent cached value
    latest_value: RefCell<Option<Value>>,

    /// If there's a notification attached to this item, this can be used to release it
    existing_notification: Option<Box<Releasable>>,

    /// What to call when the value changes
    when_changed: Vec<ReleasableNotifiable>
}

impl<Value: 'static+Clone+PartialEq, TFn> ComputedBindingCore<Value, TFn>
where TFn: 'static+Fn() -> Value {
    ///
    /// Creates a new computed binding core item
    ///
    pub fn new(calculate_value: TFn) -> ComputedBindingCore<Value, TFn> {
        ComputedBindingCore {
            calculate_value:        calculate_value,
            latest_value:           RefCell::new(None),
            existing_notification:  None,
            when_changed:           vec![]
        }
    }

    ///
    /// Marks the value as changed, returning true if the value was removed
    ///
    pub fn mark_changed(&self) -> bool {
        let mut latest_value = self.latest_value.borrow_mut();

        if *latest_value == None {
            false
        } else {
            *latest_value = None;
            true
        }
    }

    ///
    /// Retrieves a copy of the list of notifiable items for this value
    ///
    pub fn get_notifiable_items(&self) -> Vec<ReleasableNotifiable> {
        self.when_changed.clone()
    }

    ///
    /// Returns the current value (or 'None' if it needs recalculating)
    ///
    pub fn get(&self) -> Option<Value> {
        self.latest_value.borrow().clone()
    }

    ///
    /// Recalculates the latest value
    ///
    pub fn recalculate(&self) -> (Value, BindingDependencies) {
        // Perform the binding in a context to get the value and the dependencies
        let (result, dependencies) = BindingContext::bind(|| (self.calculate_value)());

        // Update the latest value
        let mut latest_value = self.latest_value.borrow_mut();
        *latest_value = Some(result.clone());

        // Pass on the result
        (result, dependencies)
    }
}

///
/// Represents a binding to a value that is computed by a function
///
#[derive(Clone)]
pub struct ComputedBinding<Value: 'static+Clone+PartialEq, TFn>
where TFn: 'static+Fn() -> Value {
    /// The core where the binding data is stored
    core: Arc<Mutex<RefCell<ComputedBindingCore<Value, TFn>>>>
}

impl<Value: 'static+Clone+PartialEq+Send, TFn> ComputedBinding<Value, TFn>
where TFn: 'static+Send+Sync+Fn() -> Value {
    ///
    /// Creates a new computable binding
    ///
    pub fn new(calculate_value: TFn) -> ComputedBinding<Value, TFn> {
        ComputedBinding {
            core: Arc::new(Mutex::new(RefCell::new(ComputedBindingCore::new(calculate_value))))
        }
    }

    ///
    /// Marks this computed binding as having changed
    ///
    fn mark_changed(&mut self) {
        // We do the notifications and releasing while the lock is not retained
        let (mut notifiable, mut releasable) = {
            // Get the core
            let lock = self.core.lock().unwrap();
            let mut core = lock.borrow_mut();

            // Mark it as changed
            let actually_changed = core.mark_changed();

            // Get the items that need changing
            let notifiable = if actually_changed {
                core.get_notifiable_items()
            } else {
                vec![]
            };

            // Extract the releasable so we can release it after the lock has gone
            let mut releasable = None;
            mem::swap(&mut releasable, &mut core.existing_notification);

            // These values are needed outside of the lock
            (notifiable, releasable)
        };

        // Don't want any more notifications from this source
        // TODO: deadlock?
        // releasable.map(|mut releasable| releasable.done());

        // Notify anything that needs to be notified that this has changed
        for to_notify in notifiable {
            to_notify.mark_as_changed();
        }
    }

    ///
    /// Mark this item as changed whenever 'to_monitor' is changed
    /// Core should already be locked
    ///
    fn monitor_changes(&self, core: &mut ComputedBindingCore<Value, TFn>, to_monitor: &mut Changeable) {
        // Clone ourselves (some weirdness in that the derived clone() function won't work here)
        let mut to_notify   = ComputedBinding { core: self.core.clone() };

        // Monitor for changes
        let lifetime        = to_monitor.when_changed(notify(move || to_notify.mark_changed()));

        // Store this as the lifetime being monitored by the core
        core.existing_notification = Some(lifetime);
    }
}

impl<Value: 'static+Clone+PartialEq, TFn> Changeable for ComputedBinding<Value, TFn>
where TFn: 'static+Send+Sync+Fn() -> Value {
    fn when_changed(&mut self, what: Arc<Notifiable>) -> Box<Releasable> {
        let releasable = ReleasableNotifiable::new(what);

        // Lock the core and push this as a thing to perform when this value changes
        let core = self.core.lock().unwrap();
        (*core.borrow_mut()).when_changed.push(releasable.clone());

        Box::new(releasable)
    }
}

impl<Value: 'static+Clone+PartialEq+Send, TFn> Bound<Value> for ComputedBinding<Value, TFn>
where TFn: 'static+Send+Sync+Fn() -> Value {
    fn get(&self) -> Value {
        // Borrow the core
        let lock = self.core.lock().unwrap();
        let mut core = lock.borrow_mut();

        if let Some(value) = core.get() {
            // The value already exists in this item
            value
        } else {
            // TODO: really want to recalculate without locking the core - can do this by moving the function out and doing the recalculation here
            // TODO: locking the core and calling a function can result in deadlocks due to user code structure in particular against other bindings
            // TODO: when we do recalculate without locking, we need to make sure that no extra invalidations arrived between when we started the calculation and when we stored the result
            // TODO: probably fine to return the out of date result rather than the newer one here

            // Stop responding to notifications
            if let Some(ref mut last_notification) = core.existing_notification {
                last_notification.done();
            }

            // Need to re-calculate the core
            let (value, mut dependencies) = core.recalculate();

            // If any of the dependencies change, mark this item as changed too
            self.monitor_changes(&mut core, &mut dependencies);

            // TODO: also need to make sure that any hooks we have are removed if we're only referenced via a hook

            // Return the value
            value
        }
    }
}

///
/// Creates a notifiable reference from a function
///
pub fn notify<TFn>(when_changed: TFn) -> Arc<Notifiable>
where TFn: 'static+Send+FnMut() -> () {
    Arc::new(NotifyFn { when_changed: Mutex::new(RefCell::new(when_changed)) })
}

///
/// Creates a simple bound value with the specified initial value
///
pub fn bind<Value: Clone+PartialEq>(val: Value) -> Binding<Value> {
    Binding::new(val)
}

pub fn computed<Value, TFn>(calculate_value: TFn) -> ComputedBinding<Value, TFn>
where Value: Clone+PartialEq+Send, TFn: 'static+Send+Sync+Fn() -> Value {
    ComputedBinding::new(calculate_value)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_binding() {
        let bound = bind(1);
        assert!(bound.get() == 1);
    }

    #[test]
    fn can_update_binding() {
        let mut bound = bind(1);

        bound.set(2);
        assert!(bound.get() == 2);
    }

    #[test]
    fn notified_on_change() {
        let mut bound   = bind(1);
        let changed     = bind(false);

        let mut notify_changed = changed.clone();
        bound.when_changed(notify(move || notify_changed.set(true)));

        assert!(changed.get() == false);
        bound.set(2);
        assert!(changed.get() == true);
    }

    #[test]
    fn not_notified_on_no_change() {
        let mut bound   = bind(1);
        let changed     = bind(false);

        let mut notify_changed = changed.clone();
        bound.when_changed(notify(move || notify_changed.set(true)));

        assert!(changed.get() == false);
        bound.set(1);
        assert!(changed.get() == false);
    }

    #[test]
    fn notifies_after_each_change() {
        let mut bound       = bind(1);
        let change_count    = bind(0);

        let mut notify_count = change_count.clone();
        bound.when_changed(notify(move || { let count = notify_count.get(); notify_count.set(count+1) }));

        assert!(change_count.get() == 0);
        bound.set(2);
        assert!(change_count.get() == 1);

        bound.set(3);
        assert!(change_count.get() == 2);

        bound.set(4);
        assert!(change_count.get() == 3);
    }

    #[test]
    fn stops_notifying_after_release() {
        let mut bound       = bind(1);
        let change_count    = bind(0);

        let mut notify_count = change_count.clone();
        let mut lifetime = bound.when_changed(notify(move || { let count = notify_count.get(); notify_count.set(count+1) }));

        assert!(change_count.get() == 0);
        bound.set(2);
        assert!(change_count.get() == 1);

        lifetime.done();
        assert!(change_count.get() == 1);
        bound.set(3);
        assert!(change_count.get() == 1);
    }

    #[test]
    fn binding_context_is_notified() {
        let mut bound = bind(1);

        bound.set(2);

        let (value, mut context) = BindingContext::bind(|| bound.get());
        assert!(value == 2);

        let changed = bind(false);
        let mut notify_changed = changed.clone();
        context.when_changed(notify(move || notify_changed.set(true)));

        assert!(changed.get() == false);
        bound.set(3);
        assert!(changed.get() == true);
    }

    #[test]
    fn can_compute_value() {
        let bound           = bind(1);

        let computed_from   = bound.clone();
        let computed        = computed(move || computed_from.get() + 1);

        assert!(computed.get() == 2);
    }

    #[test]
    fn can_recompute_value() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let computed        = computed(move || computed_from.get() + 1);

        assert!(computed.get() == 2);

        bound.set(2);
        assert!(computed.get() == 3);
    }

    #[test]
    fn computed_notifies_of_changes() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let mut computed    = computed(move || computed_from.get() + 1);

        let changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true)));

        assert!(computed.get() == 2);
        assert!(changed.get() == false);

        bound.set(2);
        assert!(changed.get() == true);
    }

    #[test]
    fn computed_stops_notifying_when_released() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let mut computed    = computed(move || computed_from.get() + 1);

        let mut changed = bind(false);
        let mut notify_changed = changed.clone();
        let mut lifetime = computed.when_changed(notify(move || notify_changed.set(true)));

        assert!(computed.get() == 2);
        assert!(changed.get() == false);

        bound.set(2);
        assert!(changed.get() == true);
        assert!(computed.get() == 3);

        changed.set(false);
        lifetime.done();

        bound.set(3);
        assert!(changed.get() == false);
        assert!(computed.get() == 4);
    }

    #[test]
    fn computed_doesnt_notify_more_than_once() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let mut computed    = computed(move || computed_from.get() + 1);

        let mut changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true)));

        assert!(computed.get() == 2);
        assert!(changed.get() == false);

        // Setting the value marks the computed as changed
        bound.set(2);
        assert!(changed.get() == true);
        changed.set(false);

        // ... but when it's already changed we don't notify again
        bound.set(3);
        assert!(changed.get() == false);

        assert!(computed.get() == 4);

        // Once we've retrieved the value, we'll get notified of changes again
        bound.set(4);
        assert!(changed.get() == true);
    }

    #[test]
    fn computed_stops_notifying_once_out_of_scope() {
        let mut bound       = bind(1);
        let mut changed     = bind(false);

        {
            let computed_from   = bound.clone();
            let mut computed    = computed(move || computed_from.get() + 1);

            let mut notify_changed = changed.clone();
            computed.when_changed(notify(move || notify_changed.set(true)));

            assert!(computed.get() == 2);
            assert!(changed.get() == false);

            bound.set(2);
            assert!(changed.get() == true);
            assert!(computed.get() == 3);
        };

        // The computed value should have been disposed of so we should get no more notifications once we reach here
        changed.set(false);
        bound.set(3);
        assert!(changed.get() == false);
    }
}

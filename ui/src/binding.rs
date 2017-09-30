//!
//! This is a binding system that provides a way to build a data-driven
//! application.
//!

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
/// Trait implemented by items that can notify something when they're changed
///
pub trait Changeable {
    ///
    /// Supplies an item to be notified when this item is changed
    ///
    fn when_changed(&mut self, what: Arc<Notifiable>);
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
    fn when_changed(&mut self, what: Arc<Notifiable>) {
        for dep in self.dependencies.borrow_mut().iter_mut() {
            dep.when_changed(what.clone());
        }
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
        let previous_context = BindingContext::current();

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
    when_changed: Vec<Arc<Notifiable>>
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
    pub fn get_notifiable_items(&self) -> Vec<Arc<Notifiable>> {
        self.when_changed.clone()
    }
}

impl<Value> Changeable for BoundValue<Value> {
    fn when_changed(&mut self, what: Arc<Notifiable>) {
        self.when_changed.push(what);
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
            for notify in self.when_changed.iter() {
                notify.mark_as_changed();
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
    fn when_changed(&mut self, what: Arc<Notifiable>) {
        let cell = self.value.lock().unwrap();
        cell.borrow_mut().when_changed(what);
    }
}

impl<Value: Clone> Bound<Value> for Binding<Value> {
    fn get(&self) -> Value {
        let cell    = self.value.lock().unwrap();
        let value   = cell.borrow().get();

        value
    }
}

impl<Value: Clone+PartialEq> MutableBound<Value> for Binding<Value> {
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
        for to_notify in notifications.into_iter() {
            to_notify.mark_as_changed()
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
}

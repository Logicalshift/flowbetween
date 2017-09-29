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
pub trait Notifiable : Send {
    ///
    /// Indicates that a dependency of this object has changed
    ///
    fn mark_as_changed(&mut self);
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
    fn get<'a>(&'a self) -> &'a Value;
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

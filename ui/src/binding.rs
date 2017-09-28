//!
//! This is a binding system that provides a way to build a data-driven
//! application.
//!

use std::sync::*;
use std::cell::*;

///
/// Trait implemented by items with dependencies that need to be notified when they have changed
///
pub trait Notifiable : Sync {
    ///
    /// Indicates that a dependency of this object has changed
    ///
    fn mark_as_changed(&mut self);
}

///
/// Trait implemented by items that can be changed
///
pub trait Changeable {
    ///
    /// Supplies an item to be notified when this item is changed
    ///
    fn when_changed(&self, what: Arc<Mutex<Notifiable>>);
}

///
/// Represents a binding context. Binding contexts are
/// per-thread structures, used to track 
///
#[derive(Clone)]
pub struct BindingContext {
    /// The item to notify if items in this context changes
    what_to_notify: Arc<Mutex<Notifiable>>,

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
    pub fn bind<TNotify, TResult, TFn>(notify: TNotify, to_do: TFn) -> TResult 
    where TNotify: 'static+Notifiable, TFn: FnOnce() -> TResult {
        // Remember the previous context
        let previous_context = BindingContext::current();

        // Create a new context
        let new_context = BindingContext {
            what_to_notify: Arc::new(Mutex::new(notify)),
            nested:         previous_context.clone().map(|ctx| Box::new(ctx))
        };

        // Make the current context the same as the new context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = Some(new_context));

        // Perform the requested action with this context
        let result = to_do();

        // Reset to the previous context
        CURRENT_CONTEXT.with(|current_context| *current_context.borrow_mut() = previous_context);

        result
    }
}

impl Changeable for BindingContext {
    fn when_changed(&self, what: Arc<Mutex<Notifiable>>) {
        unimplemented!()
    }
}

impl Changeable for Option<BindingContext> {
    fn when_changed(&self, what: Arc<Mutex<Notifiable>>) {
        self.as_ref().map(move |ctx| ctx.when_changed(what));
    }
}
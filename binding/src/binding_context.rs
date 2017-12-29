use super::traits::*;

use std::rc::*;
use std::sync::*;
use std::cell::*;

thread_local! {
    static CURRENT_CONTEXT: RefCell<Option<BindingContext>> = RefCell::new(None);
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
    fn when_changed(&self, what: Arc<Notifiable>) -> Box<Releasable> {
        let mut to_release = vec![];

        for dep in self.dependencies.borrow_mut().iter_mut() {
            to_release.push(dep.when_changed(Arc::clone(&what)));
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

impl BindingContext {
    ///
    /// Gets the active binding context
    ///
    pub fn current() -> Option<BindingContext> {
        CURRENT_CONTEXT.with(|current_context| {
            current_context
                .borrow()
                .as_ref()
                .cloned()
        })
    }

    ///
    /// Panics if we're trying to create a binding, with a particular message
    /// 
    pub fn panic_if_in_binding_context(msg: &str) {
        if CURRENT_CONTEXT.with(|context| context.borrow().is_some()) {
            panic!("Not possible when binding: {}", msg);
        }
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
            nested:         previous_context.clone().map(Box::new)
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

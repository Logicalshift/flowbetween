use super::traits::*;
use super::notify_fn::*;
use super::releasable::*;
use super::binding_context::*;

use std::sync::*;
use std::cell::*;
use std::mem;

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

impl<Value: 'static+Clone+PartialEq, TFn> Drop for ComputedBindingCore<Value, TFn>
where TFn: 'static+Fn() -> Value {
    fn drop(&mut self) {
        // No point receiving any notifications once the core has gone
        // (The notification can still fire if it has a weak reference)
        if let Some(ref mut existing_notification) = self.existing_notification {
            existing_notification.done()
        }
    }
}

///
/// Represents a binding to a value that is computed by a function
///
pub struct ComputedBinding<Value: 'static+Clone+PartialEq, TFn>
where TFn: 'static+Fn() -> Value {
    /// The core where the binding data is stored
    core: Arc<Mutex<ComputedBindingCore<Value, TFn>>>
}

impl<Value: 'static+Clone+PartialEq+Send, TFn> ComputedBinding<Value, TFn>
where TFn: 'static+Send+Sync+Fn() -> Value {
    ///
    /// Creates a new computable binding
    ///
    pub fn new(calculate_value: TFn) -> ComputedBinding<Value, TFn> {
        ComputedBinding {
            core: Arc::new(Mutex::new(ComputedBindingCore::new(calculate_value)))
        }
    }

    ///
    /// Marks this computed binding as having changed
    ///
    fn mark_changed(&mut self) {
        // We do the notifications and releasing while the lock is not retained
        let (notifiable, releasable) = {
            // Get the core
            let mut core = self.core.lock().unwrap();

            // Mark it as changed
            let actually_changed = core.mark_changed();

            // Get the items that need changing
            let notifiable = if actually_changed {
                core.get_notifiable_items()
            } else {
                vec![]
            };

            // Extract the releasable so we can release it after the lock has gone
            let mut releasable: Option<Box<Releasable>> = None;
            mem::swap(&mut releasable, &mut core.existing_notification);

            // These values are needed outside of the lock
            (notifiable, releasable)
        };

        // Don't want any more notifications from this source
        releasable.map(|mut releasable| releasable.done());

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
        // We only keep a week reference to the core here
        let to_notify   = Arc::downgrade(&self.core);

        // Monitor for changes
        let lifetime    = to_monitor.when_changed(notify(move || {
            // If the reference is still active, reconstitute a computed binding in order to call the mark_changed method
            if let Some(to_notify) = to_notify.upgrade() {
                let mut to_notify = ComputedBinding { core: to_notify };
                to_notify.mark_changed();
            } else if cfg!(debug_assertions) {
                // We can carry on here, but this suggests a memory leak
                panic!("The core of a computed is gone but its notifcations have been left behind");
            }
        }));

        // Store the lifetime
        let mut last_notification = Some(lifetime);
        mem::swap(&mut last_notification, &mut core.existing_notification);

        // Any lifetime that was in the core before this one should be finished
        last_notification.map(|mut last_notification| last_notification.done());
    }
}

impl<Value: 'static+Clone+PartialEq+Send, TFn> Clone for ComputedBinding<Value, TFn>
where TFn: 'static+Send+Sync+Fn() -> Value {
    fn clone(&self) -> Self {
        ComputedBinding { core: self.core.clone() }
    }
}

impl<Value: 'static+Clone+PartialEq, TFn> Changeable for ComputedBinding<Value, TFn>
where TFn: 'static+Send+Sync+Fn() -> Value {
    fn when_changed(&mut self, what: Arc<Notifiable>) -> Box<Releasable> {
        let releasable = ReleasableNotifiable::new(what);

        // Lock the core and push this as a thing to perform when this value changes
        let mut core = self.core.lock().unwrap();
        core.when_changed.push(releasable.clone());

        Box::new(releasable)
    }
}

impl<Value: 'static+Clone+PartialEq+Send, TFn> Bound<Value> for ComputedBinding<Value, TFn>
where TFn: 'static+Send+Sync+Fn() -> Value {
    fn get(&self) -> Value {
        // This is a dependency of the current binding context
        BindingContext::add_dependency(self.clone());

        // Borrow the core
        let mut core = self.core.lock().unwrap();

        if let Some(value) = core.get() {
            // The value already exists in this item
            value
        } else {
            // TODO: really want to recalculate without locking the core - can do this by moving the function out and doing the recalculation here
            // TODO: locking the core and calling a function can result in deadlocks due to user code structure in particular against other bindings
            // TODO: when we do recalculate without locking, we need to make sure that no extra invalidations arrived between when we started the calculation and when we stored the result
            // TODO: probably fine to return the out of date result rather than the newer one here

            // Stop responding to notifications
            let mut old_notification = None;
            mem::swap(&mut old_notification, &mut core.existing_notification);

            if let Some(mut last_notification) = old_notification {
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

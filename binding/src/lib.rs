//!
//! # Bindings
//!
//! This library provides a mechansim for designing applications with a data-driven
//! control flow. This is essentially the paradigm found in spreadsheets, where
//! a formula will update automatically if its dependencies update. This is a
//! convenient way to express how a user interface updates, as it eliminates the
//! need to describe and manage events.
//! 
//! A simple binding can be created using `let binding = bind(X)`. This creates
//! a simple mutable binding that can be updated using `binding.set(Y)` or
//! retrieved using `binding.get()`.
//! 
//! This value can be monitored using `when_changed`, as in 
//! `let lifetime = binding.when_changed(notify(|| println!("Changed!")))`.
//! The lifetime value returned can be used to stop the notifications from firing.
//! Unless `lifetime.keep_alive()` is called, it will also stop the notifications
//! once it goes out of scope.
//! 
//! So far, this is very similar to traditional observables. What makes bindings
//! different is the idea that they can be computed as well. Here's an example:
//! 
//! ```
//! # use binding::*;
//! let mut number      = bind(1);
//! let number_clone    = number.clone();
//! let plusone         = computed(move || number_clone.get() + 1);
//! 
//! let mut lifetime    = plusone.when_changed(notify(|| println!("Changed!")));
//! 
//! println!("{}", plusone.get());  // 2
//! # assert!(plusone.get() == 2);
//! number.set(2);                  // 'Changed!'
//! println!("{}", plusone.get());  // 3
//! # assert!(plusone.get() == 3);
//! lifetime.done();
//! 
//! number.set(3);                  // Lifetime is done, so no notification
//! println!("{}", plusone.get());  // 4
//! # assert!(plusone.get() == 4);
//! ```
//! 
//! Computed values can be as complicated as necessary, and will notify the 
//! specified function whenever their value changes.
//! 
//! Cloning a binding creates a new binding that references the same location.
//! This makes it easier to pass bindings into closures (though still a
//! little awkward as Rust doesn't really have a shorthand way of doing this
//! for philosophical reasons). They're similar in behaviour to an 
//! `Arc<Mutex<X>>` object in this respect (and never really immutable given
//! that cloning them creates a mutable object)
//! 
//! Computed bindings as demonstrated above are 'lazy'. They don't know their
//! own value until they have been evaluated after a change (and start in this
//! 'uncertain' state), so they will not notify of value changes until they
//! have been read at least once and will not notify again until they have
//! been read again. Reading the value from within the notification is possible
//! but in general the idea is to queue up an update for later: being lazy in
//! this way prevents repeated computations of intermediate values when many
//! values are being updated. Knock-on effects are all accounted for, so if
//! a future update is queued, it won't trigger further notifications.
//!

extern crate futures;

mod traits;
pub mod binding_context;
mod binding;
mod computed;
mod notify_fn;
mod streaming;
mod releasable;

pub use self::traits::*;
pub use self::binding::*;
pub use self::computed::*;
pub use self::notify_fn::*;
pub use self::streaming::*;

///
/// Creates a simple bound value with the specified initial value
///
pub fn bind<Value: Clone+PartialEq>(val: Value) -> Binding<Value> {
    Binding::new(val)
}

///
/// Creates a computed value that tracks bindings accessed during the function call and marks itself as changed when any of these dependencies also change
///
pub fn computed<Value, TFn>(calculate_value: TFn) -> ComputedBinding<Value, TFn>
where Value: Clone+PartialEq+Send, TFn: 'static+Send+Sync+Fn() -> Value {
    ComputedBinding::new(calculate_value)
}

#[cfg(test)]
mod test {
    use super::*;
    use super::binding_context::*;

    use std::sync::*;

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
        bound.when_changed(notify(move || notify_changed.set(true))).keep_alive();

        assert!(changed.get() == false);
        bound.set(2);
        assert!(changed.get() == true);
    }

    #[test]
    fn not_notified_on_no_change() {
        let mut bound   = bind(1);
        let changed     = bind(false);

        let mut notify_changed = changed.clone();
        bound.when_changed(notify(move || notify_changed.set(true))).keep_alive();

        assert!(changed.get() == false);
        bound.set(1);
        assert!(changed.get() == false);
    }

    #[test]
    fn notifies_after_each_change() {
        let mut bound       = bind(1);
        let change_count    = bind(0);

        let mut notify_count = change_count.clone();
        bound.when_changed(notify(move || { let count = notify_count.get(); notify_count.set(count+1) })).keep_alive();

        assert!(change_count.get() == 0);
        bound.set(2);
        assert!(change_count.get() == 1);

        bound.set(3);
        assert!(change_count.get() == 2);

        bound.set(4);
        assert!(change_count.get() == 3);
    }

    #[test]
    fn dispatches_multiple_notifications() {
        let mut bound       = bind(1);
        let change_count    = bind(0);

        let mut notify_count = change_count.clone();
        let mut notify_count2 = change_count.clone();
        bound.when_changed(notify(move || { let count = notify_count.get(); notify_count.set(count+1) })).keep_alive();
        bound.when_changed(notify(move || { let count = notify_count2.get(); notify_count2.set(count+1) })).keep_alive();

        assert!(change_count.get() == 0);
        bound.set(2);
        assert!(change_count.get() == 2);

        bound.set(3);
        assert!(change_count.get() == 4);

        bound.set(4);
        assert!(change_count.get() == 6);
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
    fn release_only_affects_one_notification() {
        let mut bound       = bind(1);
        let change_count    = bind(0);

        let mut notify_count = change_count.clone();
        let mut notify_count2 = change_count.clone();
        let mut lifetime = bound.when_changed(notify(move || { let count = notify_count.get(); notify_count.set(count+1) }));
        bound.when_changed(notify(move || { let count = notify_count2.get(); notify_count2.set(count+1) })).keep_alive();

        assert!(change_count.get() == 0);
        bound.set(2);
        assert!(change_count.get() == 2);

        bound.set(3);
        assert!(change_count.get() == 4);

        bound.set(4);
        assert!(change_count.get() == 6);

        lifetime.done();

        bound.set(5);
        assert!(change_count.get() == 7);

        bound.set(6);
        assert!(change_count.get() == 8);

        bound.set(7);
        assert!(change_count.get() == 9);
    }

    #[test]
    fn binding_context_is_notified() {
        let mut bound = bind(1);

        bound.set(2);

        let (value, context) = BindingContext::bind(|| bound.get());
        assert!(value == 2);

        let changed = bind(false);
        let mut notify_changed = changed.clone();
        context.when_changed(notify(move || notify_changed.set(true))).keep_alive();

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

        bound.set(3);
        assert!(computed.get() == 4);
    }

    #[test]
    fn can_recursively_compute_values() {
        let mut bound           = bind(1);

        let computed_from       = bound.clone();
        let computed_val        = computed(move || computed_from.get() + 1);

        let more_computed_from  = computed_val.clone();
        let more_computed       = computed(move || more_computed_from.get() + 1);

        assert!(computed_val.get() == 2);
        assert!(more_computed.get() == 3);

        bound.set(2);
        assert!(computed_val.get() == 3);
        assert!(more_computed.get() == 4);

        bound.set(3);
        assert!(computed_val.get() == 4);
        assert!(more_computed.get() == 5);
    }

    #[test]
    fn computed_only_recomputes_as_needed() {
        let mut bound           = bind(1);

        let counter             = Arc::new(Mutex::new(0));
        let compute_counter     = counter.clone();
        let computed_from       = bound.clone();
        let computed            = computed(move || {
            let mut counter = compute_counter.lock().unwrap();
            *counter = *counter + 1;

            computed_from.get() + 1
        });

        assert!(computed.get() == 2);
        {
            let counter = counter.lock().unwrap();
            assert!(counter.clone() == 1);
        }

        assert!(computed.get() == 2);
        {
            let counter = counter.lock().unwrap();
            assert!(counter.clone() == 1);
        }

        bound.set(2);
        assert!(computed.get() == 3);
        {
            let counter = counter.lock().unwrap();
            assert!(counter.clone() == 2);
        }
    }

    #[test]
    fn computed_caches_values() {
        let update_count            = bind(0);
        let mut bound               = bind(1);

        let computed_update_count   = Mutex::new(update_count.clone());
        let computed_from           = bound.clone();
        let computed                = computed(move || {
            let mut computed_update_count = computed_update_count.lock().unwrap();

            let new_update_count = computed_update_count.get() + 1;
            computed_update_count.set(new_update_count);
            computed_from.get() + 1
        });

        assert!(computed.get() == 2);
        assert!(update_count.get() == 1);

        assert!(computed.get() == 2);
        assert!(update_count.get() == 1);

        bound.set(2);
        assert!(computed.get() == 3);
        assert!(update_count.get() == 2);

        bound.set(3);
        assert!(update_count.get() == 2);
        assert!(computed.get() == 4);
        assert!(update_count.get() == 3);
    }

    #[test]
    fn computed_notifies_of_changes() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let computed        = computed(move || computed_from.get() + 1);

        let mut changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true))).keep_alive();

        assert!(computed.get() == 2);
        assert!(changed.get() == false);

        bound.set(2);
        assert!(changed.get() == true);
        assert!(computed.get() == 3);

        changed.set(false);
        bound.set(3);
        assert!(changed.get() == true);
        assert!(computed.get() == 4);
    }


    #[test]
    fn computed_switches_dependencies() {
        let mut switch      = bind(false);
        let mut val1        = bind(1);
        let mut val2        = bind(2);

        let computed_switch = switch.clone();
        let computed_val1   = val1.clone();
        let computed_val2   = val2.clone();
        let computed        = computed(move || {
            // Use val1 when switch is false, and val2 when switch is true
            if computed_switch.get() {
                computed_val2.get() + 1
            } else {
                computed_val1.get() + 1
            }
        });

        let mut changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true))).keep_alive();

        // Initial value of computed (first get 'arms' when_changed too)
        assert!(computed.get() == 2);
        assert!(changed.get() == false);

        // Setting val2 shouldn't cause computed to become 'changed' initially
        val2.set(3);
        assert!(changed.get() == false);
        assert!(computed.get() == 2);

        // ... but setting val1 should
        val1.set(2);
        assert!(changed.get() == true);
        assert!(computed.get() == 3);

        // Flicking the switch will use the val2 value we set earlier
        changed.set(false);
        switch.set(true);
        assert!(changed.get() == true);
        assert!(computed.get() == 4);

        // Updating val2 should now mark us as changed
        changed.set(false);
        val2.set(4);
        assert!(changed.get() == true);
        assert!(computed.get() == 5);
        
        // Updating val1 should not mark us as changed
        changed.set(false);
        val1.set(5);
        assert!(changed.get() == false);
        assert!(computed.get() == 5);
    }

    #[test]
    fn computed_propagates_changes() {
        let mut bound           = bind(1);

        let computed_from       = bound.clone();
        let propagates_from     = computed(move || computed_from.get() + 1);
        let computed_propagated = propagates_from.clone();
        let computed            = computed(move || computed_propagated.get() + 1);

        let mut changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true))).keep_alive();

        assert!(propagates_from.get() == 2);
        assert!(computed.get() == 3);
        assert!(changed.get() == false);

        bound.set(2);
        assert!(propagates_from.get() == 3);
        assert!(computed.get() == 4);
        assert!(changed.get() == true);

        changed.set(false);
        bound.set(3);
        assert!(changed.get() == true);
        assert!(propagates_from.get() == 4);
        assert!(computed.get() == 5);
    }

    #[test]
    fn computed_stops_notifying_when_released() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let computed        = computed(move || computed_from.get() + 1);

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

        bound.set(4);
        assert!(changed.get() == false);
        assert!(computed.get() == 5);
    }

    #[test]
    fn computed_doesnt_notify_more_than_once() {
        let mut bound       = bind(1);

        let computed_from   = bound.clone();
        let computed        = computed(move || computed_from.get() + 1);

        let mut changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true))).keep_alive();

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
            let computed        = computed(move || computed_from.get() + 1);

            let mut notify_changed = changed.clone();
            computed.when_changed(notify(move || notify_changed.set(true))).keep_alive();

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

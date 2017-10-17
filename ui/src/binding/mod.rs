//!
//! # Bindings
//!
//! This provides a means for building data-driven applications. The
//! basic model is similar to how spreadsheets work: we watch what
//! items a particular calculation depends on and generate an event
//! when any of these change.
//!

pub mod traits;
pub mod binding_context;
pub mod binding;
pub mod computed;
pub mod notify_fn;
mod releasable;

pub use self::traits::*;
pub use self::binding::*;
pub use self::computed::*;
pub use self::notify_fn::*;

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
    use super::super::binding_context::*;

    use std::sync::*;
    use std::cell::*;

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
    fn dispatches_multiple_notifications() {
        let mut bound       = bind(1);
        let change_count    = bind(0);

        let mut notify_count = change_count.clone();
        let mut notify_count2 = change_count.clone();
        bound.when_changed(notify(move || { let count = notify_count.get(); notify_count.set(count+1) }));
        bound.when_changed(notify(move || { let count = notify_count2.get(); notify_count2.set(count+1) }));

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
        bound.when_changed(notify(move || { let count = notify_count2.get(); notify_count2.set(count+1) }));

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

        let counter             = Arc::new(Mutex::new(RefCell::new(0)));
        let compute_counter     = counter.clone();
        let computed_from       = bound.clone();
        let computed            = computed(move || {
            let counter     = compute_counter.lock().unwrap();
            let mut counter = counter.borrow_mut();
            *counter = *counter + 1;

            computed_from.get() + 1
        });

        assert!(computed.get() == 2);
        {
            let counter = counter.lock().unwrap();
            assert!(counter.borrow().clone() == 1);
        }

        assert!(computed.get() == 2);
        {
            let counter = counter.lock().unwrap();
            assert!(counter.borrow().clone() == 1);
        }

        bound.set(2);
        assert!(computed.get() == 3);
        {
            let counter = counter.lock().unwrap();
            assert!(counter.borrow().clone() == 2);
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
        let mut computed    = computed(move || computed_from.get() + 1);

        let mut changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true)));

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
    fn computed_propagates_changes() {
        let mut bound           = bind(1);

        let computed_from       = bound.clone();
        let propagates_from     = computed(move || computed_from.get() + 1);
        let computed_propagated = propagates_from.clone();
        let mut computed        = computed(move || computed_propagated.get() + 1);

        let mut changed = bind(false);
        let mut notify_changed = changed.clone();
        computed.when_changed(notify(move || notify_changed.set(true)));

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

        bound.set(4);
        assert!(changed.get() == false);
        assert!(computed.get() == 5);
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

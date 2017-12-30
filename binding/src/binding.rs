use super::traits::*;
use super::releasable::*;
use super::binding_context::*;

use std::sync::*;

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
        self.when_changed
            .iter()
            .map(|item| item.clone_for_inspection())
            .collect()
    }

    ///
    /// If there are any notifiables in this object that aren't in use, remove them
    ///
    pub fn filter_unused_notifications(&mut self) {
        self.when_changed.retain(|releasable| releasable.is_in_use());
    }

    ///
    /// Retrieves the value of this item
    ///
    fn get(&self) -> Value {
        self.value.clone()
    }

    ///
    /// Adds something that will be notified when this item changes
    ///
    fn when_changed(&mut self, what: Arc<Notifiable>) -> Box<Releasable> {
        let releasable = ReleasableNotifiable::new(what);
        self.when_changed.push(releasable.clone_as_owned());

        self.filter_unused_notifications();

        Box::new(releasable)
    }
}

///
/// Represents a thread-safe, sharable binding
///
#[derive(Clone)]
pub struct Binding<Value> {
    /// The value stored in this binding
    value: Arc<Mutex<BoundValue<Value>>>
}

impl<Value: Clone+PartialEq> Binding<Value> {
    pub fn new(value: Value) -> Binding<Value> {
        Binding {
            value: Arc::new(Mutex::new(BoundValue::new(value)))
        }
    }
}

impl<Value: 'static+Clone+PartialEq+Send> Changeable for Binding<Value> {
    fn when_changed(&self, what: Arc<Notifiable>) -> Box<Releasable> {
        self.value.lock().unwrap().when_changed(what)
    }
}

impl<Value: 'static+Clone+PartialEq+Send> Bound<Value> for Binding<Value> {
    fn get(&self) -> Value {
        BindingContext::add_dependency(self.clone());

        self.value.lock().unwrap().get()
    }
}

impl<Value: 'static+Clone+PartialEq+Send> MutableBound<Value> for Binding<Value> {
    fn set(&mut self, new_value: Value) {
        // Update the value with the lock held
        let notifications = {
            let mut cell    = self.value.lock().unwrap();
            let changed     = cell.set_without_notifying(new_value);
        
            if changed {
                cell.get_notifiable_items()
            } else {
                vec![]
            }
        };

        // Call the notifications outside of the lock
        let mut needs_filtering = false;

        for to_notify in notifications {
            needs_filtering = !to_notify.mark_as_changed() || needs_filtering;
        }

        if needs_filtering {
            let mut cell = self.value.lock().unwrap();
            cell.filter_unused_notifications();
        }
    }
}

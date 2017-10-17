use std::sync::*;

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
    /// This event is only fired after the value has been read since the most recent
    /// change. Note that this means if the value is never read, this event may
    /// never fire. This behaviour is desirable when deferring updates as it prevents
    /// large cascades of 'changed' events occurring for complicated dependency trees.
    ///
    fn when_changed(&mut self, what: Arc<Notifiable>) -> Box<Releasable>;
}

///
/// Trait implemented by something that is bound to a value
///
pub trait Bound<Value> : Changeable+Send+Sync {
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

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
/// Trait implemented by an object that can be released: for example to stop performing
/// an action when it's no longer required.
///
pub trait Releasable : Send {
    ///
    /// Indicates that this object should not be released on drop
    ///
    fn keep_alive(&mut self);

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
    /// Supplies a function to be notified when this item is changed
    /// 
    /// This event is only fired after the value has been read since the most recent
    /// change. Note that this means if the value is never read, this event may
    /// never fire. This behaviour is desirable when deferring updates as it prevents
    /// large cascades of 'changed' events occurring for complicated dependency trees.
    /// 
    /// The releasable that's returned has keep_alive turned off by default, so
    /// be sure to store it in a variable or call keep_alive() to keep it around
    /// (if the event never seems to fire, this is likely to be the problem)
    ///
    fn when_changed(&self, what: Arc<Notifiable>) -> Box<Releasable>;
}

///
/// Trait implemented by something that is bound to a value
///
pub trait Bound<Value> : Changeable+Send+Sync {
    ///
    /// Retrieves the value stored by this binding
    ///
    fn get(&self) -> Value;

    ///
    /// Creates a boxed clone of this item
    ///
    fn clone_box(&self) -> Box<Bound<Value>>;
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

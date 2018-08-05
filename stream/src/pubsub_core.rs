use futures::task::Task;

use std::collections::VecDeque;

///
/// The core shared structure between a publisher and subscriber
/// 
pub struct PubSubCore<Message> {
    /// Unique ID for the subscriber represented by this core
    id: usize,

    /// True while the subscriber owning this core is alive
    subscribed: bool,

    /// Messages ready to be sent to this core
    waiting: VecDeque<Message>

    /// Notification task for when the 'waiting' queue becomes non-empty
    notify: Option<Task>
}

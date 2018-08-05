use super::pubsub_core::*;

use std::sync::*;

///
/// Represents a subscriber stream from a publisher sink
/// 
pub struct Subscriber<Message> {
    /// The publisher core (shared between all subscribers)
    pub_core: Weak<Mutex<PubCore<Message>>>,

    /// The subscriber core (used only by this subscriber)
    sub_core: Arc<Mutex<SubCore<Message>>>
}

impl<Message> Subscriber<Message> {
    ///
    /// Creates a new subscriber
    /// 
    pub (crate) fn new(pub_core: Weak<Mutex<PubCore<Message>>>, sub_core: Arc<Mutex<SubCore<Message>>>) -> Subscriber<Message> {
        Subscriber {
            pub_core,
            sub_core
        }
    }
}
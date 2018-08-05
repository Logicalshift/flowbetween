use super::subscriber::*;
use super::publisher_sink::*;

use futures::executor::Spawn;

pub trait PubSubSpawn<Message: Clone> {
    ///
    /// Creates a new subscriber for this publisher
    /// 
    fn subscribe(&mut self) -> Subscriber<Message>;
}

///
/// For convenience, makes it possible to subscribe() to spawned publishers without having to call get_mut()
/// 
impl<Message: Clone, S: PublisherSink<Message>> PubSubSpawn<Message> for Spawn<S> {
    fn subscribe(&mut self) -> Subscriber<Message> {
        self.get_mut().subscribe()
    }
}

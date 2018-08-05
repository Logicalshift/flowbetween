use super::publisher::*;

///
/// A blocking publisher is a publisher that blocks messages until it has enough subscribers
/// 
/// This is useful for cases where a publisher is being used asynchronously and wants to ensure that
/// its messages are sent to at least one subscriber
/// 
pub struct BlockingPublisher<Message> {
    /// The number of required subscribers
    required_subscribers: usize,

    /// The publisher where messages will be relayed
    publisher: Publisher<Message>
}

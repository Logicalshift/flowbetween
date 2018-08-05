use super::subscriber::*;
use futures::*;

///
/// Trait implemented by sinks that act as a publisher
/// 
pub trait PublisherSink<Message> : Sink<SinkItem=Message, SinkError=()> {
    ///
    /// Creates a subscription to this publisher
    /// 
    /// Any future messages sent here will also be sent to this subscriber.
    /// 
    fn subscribe(&mut self) -> Subscriber<Message>;
}

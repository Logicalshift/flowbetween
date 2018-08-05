use super::subscriber::*;

use futures::*;
use futures::sink::Sink;

use std::collections::VecDeque;

///
/// A publisher represents a sink that sends messages to zero or more subscribers
/// 
pub struct Publisher<Message> {
    /// The maximum number of messages to buffer
    max_buffer_size: usize,

    /// The buffer for this publisher
    buffer: VecDeque<Message>
}

impl<Message> Publisher<Message> {
    ///
    /// Creates a new publisher with a particular buffer size
    /// 
    pub fn new(buffer_size: usize) -> Publisher<Message> {
        Publisher {
            max_buffer_size:    buffer_size,
            buffer:             VecDeque::new()
        }
    }

    ///
    /// Subscribes to this publisher
    /// 
    /// The first subscriber will receive all messages that have been queued before it was created. Future subscribers
    /// will only receive messages that are sent after it is created.
    /// 
    pub fn subscribe(&mut self) -> Subscriber<Message> {
        unimplemented!()        
    }
}

impl<Message> Sink for Publisher<Message> {
    type SinkItem   = Message;
    type SinkError  = ();

    fn start_send(&mut self, item: Message) -> StartSend<Message, ()> {
        unimplemented!()
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        unimplemented!()
    }
}

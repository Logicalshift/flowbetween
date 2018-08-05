use std::marker::PhantomData;

///
/// Represents a subscriber stream from a publisher sink
/// 
pub struct Subscriber<Message> {
    message: PhantomData<Message>
}

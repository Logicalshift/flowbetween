use futures::sink::*;
use futures::stream::*;

///
/// Trait that can be implemented by items that represent a user interface
/// 
pub trait UserInterface<InputEvent, OutputUpdate, Error> {
    /// The type of the event sink for this UI
    type EventSink: Sink<SinkItem = InputEvent, SinkError = ()>;

    /// The type of the update stream for this UI
    type UpdateStream: Stream<Item = OutputUpdate, Error = Error>;

    /// Retrieves an input event sink for this user interface
    fn get_input_sink(&self) -> Self::EventSink;

    /// Retrieves a view onto the update stream for this user interface
    fn get_updates(&self) -> Self::UpdateStream;
}

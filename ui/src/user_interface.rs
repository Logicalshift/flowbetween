use super::controller::*;

use futures::sink::*;
use futures::stream::*;

use std::sync::*;

///
/// Trait that can be implemented by items that represent a user interface
/// 
pub trait UserInterface<InputEvent, OutputUpdate, Error> {
    /// The type of the event sink for this UI
    type EventSink: Sink<SinkItem = InputEvent, SinkError = ()>;

    /// The type of the update stream for this UI
    type UpdateStream: Stream<Item = OutputUpdate, Error = Error>;

    /// The type of the main controller for this user interface
    type CoreController: Controller;

    /// Retrieves an input event sink for this user interface
    fn get_input_sink(&self) -> Self::EventSink;

    /// Retrieves a view onto the update stream for this user interface
    fn get_updates(&self) -> Self::UpdateStream;

    /// Retrieves the core controller for this UI
    fn controller(&self) -> Arc<Self::CoreController>;
}

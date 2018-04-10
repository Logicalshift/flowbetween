use flo_ui::session::*;

use futures::*;

///
/// Stream that takes a stream of UiEvents destined for the core UI and consolidates any buffered actions that can be considered as a single
/// action
/// 
pub struct ConsolidateActionsStream<ActionStream> {
    /// The stream that this will read from
    source_stream: ActionStream,

    /// A future event we unbuffered but could not consolidate with the current event
    pending_event: Option<Vec<UiEvent>>
}

impl<ActionStream: Stream<Item=Vec<UiEvent>, Error=()>> ConsolidateActionsStream<ActionStream> {
    ///
    /// Creates a new consolidating stream
    /// 
    pub fn new(source_stream: ActionStream) -> ConsolidateActionsStream<ActionStream> {
        ConsolidateActionsStream {
            source_stream: source_stream,
            pending_event: None
        }
    }

    ///
    /// Attempts to consolidate an event with a future event
    /// 
    fn consolidate(&mut self, next_event: Vec<UiEvent>, future_event: Poll<Option<Vec<UiEvent>>, ()>) -> (Vec<UiEvent>, Poll<Option<Vec<UiEvent>>, ()>) {
        use self::Async::*;

        if let Ok(Ready(Some(future_event))) = future_event {
            let mut next_event = next_event;
            next_event.extend(future_event);

            (next_event, self.source_stream.poll())
        } else {
            (next_event, future_event)
        }
    }
}

impl<ActionStream: Stream<Item=Vec<UiEvent>, Error=()>> Stream for ConsolidateActionsStream<ActionStream> {
    type Item=Vec<UiEvent>;
    type Error=();

    fn poll(&mut self) -> Poll<Option<Vec<UiEvent>>, ()> {
        use self::Async::*;

        if let Some(pending_event) = self.pending_event.take() {
            // If there's already a pending event, this is what we return
            Ok(Ready(Some(pending_event)))
        } else {
            // Try to fetch the next event from the source stream
            let mut next_event = self.source_stream.poll();

            if let Ok(Ready(Some(mut next_event))) = next_event {
                // An event is ready: see if another event is immediately available
                let mut future_event = self.source_stream.poll();

                // Loop until there are no more future events to consolidate
                loop {
                    match future_event {
                        Ok(Ready(Some(_)))      => { let (new_next_event, new_future_event) = self.consolidate(next_event, future_event); next_event = new_next_event; future_event = new_future_event; },
                        _                       => { break; }
                    }
                }

                Ok(Ready(Some(next_event)))
            } else {
                // Result is just the next event
                next_event
            }
        }
    }
}
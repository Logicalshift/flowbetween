use futures::*;
use futures::task::{Poll, Context};

use std::pin::*;

// TODO: this probably belongs somewhere like flo_streams

///
/// Stream that takes a stream of `Vec`s and gathers as many as possible into single items
/// 
/// (That is, if there are two sets of items immediately available on the source stream, this will return 
/// both)
///
pub struct GatherStream<SourceStream> {
    /// The source stream, or none if it has finished
    source_stream: Option<SourceStream>
}

///
/// Creates a new 'gather' stream that collects items together into a longer stream
///
pub fn gather<SourceStream, SourceItem>(source: SourceStream) -> GatherStream<SourceStream>
where SourceStream: Stream<Item=Vec<SourceItem>>+Unpin {
    GatherStream {
        source_stream: Some(source)
    }
}

impl<SourceStream, SourceItem> Stream for GatherStream<SourceStream> 
where   SourceStream: Stream<Item=Vec<SourceItem>>+Unpin {
    type Item = SourceStream::Item;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Vec<SourceItem>>> {
        if let Some(source_stream) = self.source_stream.as_mut() {
            // Attempt to gather items from the source stream
            let poll_result     = source_stream.poll_next_unpin(context);

            match poll_result {
                Poll::Ready(Some(mut items)) => {
                    // Try to gather more items into the result
                    loop {
                        let poll_result     = source_stream.poll_next_unpin(context);

                        match poll_result {
                            // Retrieved more items
                            Poll::Ready(Some(more_items))   => { items.extend(more_items); },

                            // End of the stream
                            Poll::Ready(None)               => { self.source_stream = None; break; },

                            // No more items waiting on the stream
                            Poll::Pending                   => { break; }
                        }
                    }
                    
                    // Result is the items we gathered
                    Poll::Ready(Some(items))
                }

                Poll::Ready(None) => {
                    // Fuse the stream
                    self.source_stream = None;
                    Poll::Ready(None)
                }

                Poll::Pending => {
                    // No items are available yet
                    Poll::Pending
                }
            }

        } else {
            // Stream has finished
            Poll::Ready(None)
        }
    }
}

use futures::*;
use futures::stream;
use futures::task::{Poll};

///
/// Given a stream of Vecs<>, tries to read as many entries as possible from the stream and collapses them into
/// single Vec<> items. This groups things together when the stream is generating very many items.
///
pub fn group_stream<Item, ItemStream: Stream<Item=Result<Vec<Item>, ()>>+Unpin>(stream: ItemStream) -> impl Stream<Item=Result<Vec<Item>, ()>> {
    // Fusing the stream means we can always safely read from it even after it has finished
    let mut stream = stream.fuse();

    stream::poll_fn(move |context| {
        // The items grouped into the current event
        let mut grouped_items: Option<Vec<Item>> = None;

        loop {
            // Read from the current stream until it returns not ready (or finished)
            let next = stream.poll_next_unpin(context);

            match next {
                Poll::Ready(None) => {
                    // End of stream
                    if let Some(grouped_items) = grouped_items {
                        // Still some items waiting to return
                        return Poll::Ready(Some(Ok(grouped_items)));
                    } else {
                        // No items waiting
                        return Poll::Ready(None);
                    }
                },

                Poll::Ready(Some(Ok(items))) => {
                    if let Some(mut existing_group) = grouped_items {
                        // Add to the existing group set
                        existing_group.extend(items);
                        grouped_items = Some(existing_group);
                    } else {
                        // Create a new group from these items
                        grouped_items = Some(items);
                    }
                }

                Poll::Pending => {
                    // No more items to fetch
                    if let Some(grouped_items) = grouped_items {
                        // Return what we got so far from the stream
                        return Poll::Ready(Some(Ok(grouped_items)));
                    } else {
                        // No items waiting to send
                        return Poll::Pending;
                    }
                }

                Poll::Ready(Some(Err(_))) => {
                    return Poll::Ready(Some(Err(())));
                }
            }
        }
    })
}

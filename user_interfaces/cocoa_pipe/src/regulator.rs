use futures::*;
use futures::stream;

///
/// Given a stream of Vecs<>, tries to read as many entries as possible from the stream and collapses them into
/// single Vec<> items. This groups things together when the stream is generating very many items.
///
pub fn group_stream<Item, ItemStream: Stream<Item=Vec<Item>, Error=()>>(stream: ItemStream) -> impl Stream<Item=Vec<Item>, Error=()> {
    // Fusing the stream means we can always safely read from it even after it has finished
    let mut stream = stream.fuse();

    stream::poll_fn(move || {
        // The items grouped into the current event
        let mut grouped_items: Option<Vec<Item>> = None;

        loop {
            // Read from the current stream until it returns not ready (or finished)
            let next = stream.poll();

            match next {
                Ok(Async::Ready(None)) => {
                    // End of stream
                    if let Some(grouped_items) = grouped_items {
                        // Still some items waiting to return
                        return Ok(Async::Ready(Some(grouped_items)));
                    } else {
                        // No items waiting
                        return Ok(Async::Ready(None));
                    }
                },

                Ok(Async::Ready(Some(items))) => {
                    if let Some(mut existing_group) = grouped_items {
                        // Add to the existing group set
                        existing_group.extend(items);
                        grouped_items = Some(existing_group);
                    } else {
                        // Create a new group from these items
                        grouped_items = Some(items);
                    }
                }

                Ok(Async::NotReady) => {
                    // No more items to fetch
                    if let Some(grouped_items) = grouped_items {
                        // Return what we got so far from the stream
                        return Ok(Async::Ready(Some(grouped_items)));
                    } else {
                        // No items waiting to send
                        return Ok(Async::NotReady);
                    }
                }

                Err(_) => {
                    return Err(())
                }
            }
        }
    })
}

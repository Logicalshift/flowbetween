```toml
flo_stream = "0.1"
```

# flo_stream

`flo_stream` is a crate providing some extra utilities for streams in Rust's `futures` library. The primary new feature
it provides is a "pubsub" mechanism - a way to subscribe to updates sent to a futures `Sink`. This differs from the
`Sender`/`Receiver` mechanism provided in the main futures library in two key ways: it's possible to have multiple
receivers, and messages sent when there is no subscriber connected will be ignored.

## PubSub

The sink type provided is `Publisher`. You can create one with `let publisher = Publisher::new(10)`. This implements 
the `Sink` trait so can be used in a very similar way to send messages. The number passed in is the maximum number
of waiting messages allowed for any given subscriber.

A subscription can be created using `let subscription = publisher.subscribe()`. Any messages sent to the sink after
this is called is relayed to all subscriptions. A subscription is a `Stream` so can interact with other parts of the
futures library in the usual way.

Here's a full worked example with a single subscriber. Note the use of `get_mut()` with the spawned executor to create 
a new subscriber:

```Rust
let publisher       = Publisher::new(10);
let mut publisher   = executor::spawn(publisher);

let subscriber      = publisher.get_mut().subscribe();
let mut subscriber  = executor::spawn(subscriber);

publisher.wait_send(1).unwrap();
publisher.wait_send(2).unwrap();
publisher.wait_send(3).unwrap();

assert!(subscriber.wait_stream() == Some(Ok(1)));
assert!(subscriber.wait_stream() == Some(Ok(2)));
assert!(subscriber.wait_stream() == Some(Ok(3)));
```

It's also possible to call `subscriber.clone()` to create a new subscription from an existing one without needing to 
keep a reference to the publisher. This can be used to reduce the amount of effort needed in passing objects around, and
to hide implementation details from the caller.

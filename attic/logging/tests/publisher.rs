extern crate flo_logging;
extern crate desync;

use flo_logging::*;
use futures::future;
use desync::*;

use std::sync::*;
use std::thread;
use std::time::Duration;

#[test]
fn publish_log_messages_to_subscriber() {
    let log         = LogPublisher::new("test");
    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), log.subscribe(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    thread::sleep(Duration::from_millis(10));

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(messages[0].message() == "Hello, world");
    assert!(messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}

#[test]
fn publish_log_messages_to_two_subscribers() {
    let log         = LogPublisher::new("test");
    let messages1   = Arc::new(Desync::new(vec![]));
    let messages2   = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages1), log.subscribe(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });
    pipe_in(Arc::clone(&messages2), log.subscribe(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    thread::sleep(Duration::from_millis(10));

    let messages    = messages1.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(messages[0].message() == "Hello, world");
    assert!(messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);

    let messages    = messages2.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(messages[0].message() == "Hello, world");
    assert!(messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}

#[test]
fn log_message_message_set_properly() {
    let log         = LogPublisher::new("test");
    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), log.subscribe(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    thread::sleep(Duration::from_millis(10));

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(messages[0].fields().iter().any(|(field_name, field_msg)| field_name == "message" && field_msg == "Hello, world"));
    assert!(messages.len() == 2);
}

#[test]
fn log_message_target_set_properly() {
    let log         = LogPublisher::new("test");
    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), log.subscribe(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    thread::sleep(Duration::from_millis(10));

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(messages[0].fields().iter().any(|(field_name, field_msg)| field_name == "target" && field_msg == "test"));
    assert!(messages.len() == 2);
}

#[test]
fn publish_log_messages_to_default() {
    let log         = LogPublisher::new("test");
    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), log.subscribe_default(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    thread::sleep(Duration::from_millis(10));

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(messages[0].message() == "Hello, world");
    assert!(messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}

#[test]
fn no_messages_to_default_with_subscriber() {
    let log                 = LogPublisher::new("test");
    let messages_default    = Arc::new(Desync::new(vec![]));
    let messages_nondefault = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages_default), log.subscribe_default(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });
    pipe_in(Arc::clone(&messages_nondefault), log.subscribe(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    thread::sleep(Duration::from_millis(10));

    let messages_nondefault = messages_nondefault.sync(|messages_nondefault| messages_nondefault.clone());
    let messages_default    = messages_default.sync(|messages_default| messages_default.clone());

    assert!(messages_nondefault.len() != 0);
    assert!(messages_nondefault[0].message() == "Hello, world");
    assert!(messages_nondefault[1].message() == "... goodbye, world :-(");
    assert!(messages_nondefault.len() == 2);

    assert!(messages_default.len() == 0);
}

#[test]
fn stream_between_logs() {
    let src         = LogPublisher::new("testSrc");
    let tgt         = LogPublisher::new("testTgt");
    let messages    = Arc::new(Desync::new(vec![]));

    // Result is messages from target
    pipe_in(Arc::clone(&messages), tgt.subscribe(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });

    // Target relays logs from src
    tgt.stream(src.subscribe());

    // Send messages to src
    src.log("Hello, world");
    src.log("... goodbye, world :-(");

    thread::sleep(Duration::from_millis(20));   // TODO: arrange things so that propagation is instant somehow
    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(messages[0].message() == "Hello, world");
    assert!(messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}

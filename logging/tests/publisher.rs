extern crate flo_logging;
extern crate desync;

use flo_logging::*;
use desync::*;

use std::sync::*;

#[test]
fn publish_log_messages_to_subscriber() {
    let log         = LogPublisher::new();
    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), log.subscribe(), |messages, new_message| messages.push(new_message.unwrap()));

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(&messages[0].message() == "Hello, world");
    assert!(&messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}

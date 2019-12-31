extern crate flo_logging;
extern crate desync;
#[macro_use] extern crate log;

use flo_logging::*;
use futures::future;
use desync::*;

use std::sync::*;
use std::thread;
use std::time::Duration;

#[test]
fn rust_log_to_flo_log() {
    send_rust_logs_to_flo_logs().unwrap();

    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), subscribe_to_logs(), |messages, new_message| { messages.push(new_message); Box::pin(future::ready(())) });

    info!("Hello, world");

    thread::sleep(Duration::from_millis(20));

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(messages[0].message() == "Hello, world");
    assert!(messages.len() == 1);
}

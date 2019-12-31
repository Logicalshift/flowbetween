use super::log_msg::*;
use super::message::*;

use log;
use desync::{Desync, pipe_in};
use futures::*;

use std::sync::*;

lazy_static! {
    ///
    /// Desync that relays our log messages for us
    ///
    static ref LOGGER: Arc<Desync<()>> = Arc::new(Desync::new(()));
}

///
/// Subscribes log messages on a particular stream to the log framework
///
pub fn send_to_stderr<LogStream: 'static+Unpin+Send+Stream<Item=LogMsg>>(stream: LogStream) {
    pipe_in(Arc::clone(&LOGGER), stream, |_, msg| {
        let message = msg.message();
        let target  = msg.field_value("target").unwrap_or("");

        match msg.level() {
            log::Level::Trace   => { eprintln!("TRACE   {}: {}", target, message); },
            log::Level::Debug   => { eprintln!("DEBUG   {}: {}", target, message); },
            log::Level::Info    => { eprintln!("INFO    {}: {}", target, message); },
            log::Level::Warn    => { eprintln!("WARNING {}: {}", target, message); },
            log::Level::Error   => { eprintln!("ERROR   {}: {}", target, message); }
        }

        Box::pin(future::ready(()))
    });
}

use super::level::*;
use super::log_msg::*;
use super::message::*;

use desync::*;
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
pub fn send_to_stderr<LogStream: 'static+Send+Stream<Item=LogMsg, Error=()>>(stream: LogStream) {
    pipe_in(Arc::clone(&LOGGER), stream, |_, msg| {
        msg.map(|msg| {
            let message = msg.message();
            let target  = msg.field_value("target").unwrap_or("");

            match msg.level() {
                LogLevel::Debug     => { eprintln!("DEBUG    {}: {}", target, message); },
                LogLevel::Verbose   => { eprintln!("VERBOSE  {}: {}", target, message); },
                LogLevel::Info      => { eprintln!("INFO     {}: {}", target, message); },
                LogLevel::Warning   => { eprintln!("WARNING  {}: {}", target, message); },
                LogLevel::Error     => { eprintln!("ERROR    {}: {}", target, message); },
                LogLevel::Critical  => { eprintln!("CRITICAL {}: {}", target, message); },
            }
        }).ok();
    });
}

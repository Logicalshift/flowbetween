use super::level::*;
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
pub fn send_to_logs<Msg: LogMessage, LogStream: 'static+Send+Stream<Item=Msg, Error=()>>(stream: LogStream) {
    pipe_in(Arc::clone(&LOGGER), stream, |_, msg| {
        msg.map(|msg| {
            let message = msg.message();

            match msg.level() {
                LogLevel::Debug     => { trace!("{}", message); },
                LogLevel::Verbose   => { debug!("{}", message); },
                LogLevel::Info      => { info!("{}", message); },
                LogLevel::Warning   => { warn!("{}", message); },
                LogLevel::Error     => { error!("{}", message); },
                LogLevel::Critical  => { error!("{}", message); }
            }
        }).ok();
    });
}

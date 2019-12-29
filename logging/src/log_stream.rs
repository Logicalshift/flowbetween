use super::log_msg::*;
use super::static_log::*;

use futures::*;

///
/// Subscribes to the messages generated in the current logging context
///
pub fn subscribe_to_logs() -> impl Stream<Item=LogMsg> {
    current_log().subscribe()
}

use super::publisher::*;
use super::log_subscriber::*;

use std::cell::*;

thread_local! {
    static THREAD_LOGGER: RefCell<Option<LogPublisher>> = RefCell::new(None);
}

///
/// Retrieves a reference to the log publisher for the current thread
/// 
pub fn log() -> LogPublisher {
    THREAD_LOGGER.with(|logger| {
        let mut logger = logger.borrow_mut();

        if logger.is_some() {
            // Existing logger
            logger.as_ref().unwrap().clone()
        } else {
            // New logger
            let new_logger = LogPublisher::new_empty();
            
            // Default to printing the message (via the println logger desync object)
            send_to_logs(new_logger.subscribe_default());

            *logger = Some(new_logger);
            logger.as_ref().unwrap().clone()
        }
    })
}

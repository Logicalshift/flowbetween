use super::publisher::*;

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
            *logger = Some(LogPublisher::new());
            logger.as_ref().unwrap().clone()
        }
    })
}

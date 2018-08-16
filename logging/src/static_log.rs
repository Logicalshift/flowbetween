use super::message::*;
use super::publisher::*;

use desync::*;

use std::cell::*;
use std::sync::*;

thread_local! {
    static THREAD_LOGGER: RefCell<Option<LogPublisher>> = RefCell::new(None);
}

lazy_static! {
    static ref PRINTLN_LOGGER: Arc<Desync<()>> = Arc::new(Desync::new(()));
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
            let new_logger = LogPublisher::new();
            
            // Default to printing the message (via the println logger desync object)
            pipe_in(Arc::clone(&PRINTLN_LOGGER), new_logger.subscribe_default(), |_, log| { log.map(|log| println!("{}", log.message())).ok(); });

            *logger = Some(new_logger);
            logger.as_ref().unwrap().clone()
        }
    })
}

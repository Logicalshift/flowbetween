use super::publisher::*;
use super::log_subscriber::*;

use std::cell::*;

thread_local! {
    static THREAD_LOGGER: RefCell<Option<LogPublisher>> = RefCell::new(None);
}

lazy_static! {
    static ref CORE_LOGGER: LogPublisher = LogPublisher::new_core_publisher();
}

impl LogPublisher {
    ///
    /// Creates a new 'core' log publisher
    /// 
    fn new_core_publisher() -> LogPublisher {
        // Create a new logger
        let new_logger = LogPublisher::new_empty(vec![("target", "flo_logger::core")]);

        // Default messages here get the default behaviour
        send_to_stderr(new_logger.subscribe_default());

        new_logger
    }
}

///
/// Retrieves a reference to the log publisher for the current thread
/// 
pub fn current_log() -> LogPublisher {
    THREAD_LOGGER.with(|logger| {
        let mut logger = logger.borrow_mut();

        if logger.is_some() {
            // Existing logger
            logger.as_ref().unwrap().clone()
        } else {
            // Start with the core logger
            let core_logger = CORE_LOGGER.clone();
            *logger = Some(core_logger);

            logger.as_ref().unwrap().clone()
        }
    })
}

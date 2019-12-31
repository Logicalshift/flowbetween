use super::message::*;
use super::publisher::*;
use super::log_subscriber::*;

use log;
use log::*;
use futures::future;
use desync::{Desync, pipe_in};

use std::sync::*;
use std::cell::*;
use std::collections::HashMap;

thread_local! {
    static THREAD_LOGGER: RefCell<Option<LogPublisher>> = RefCell::new(None);
}

lazy_static! {
    static ref CORE_LOGGER: LogPublisher = LogPublisher::new_core_publisher();
    static ref CORE_PROCESSOR: Arc<Desync<()>> = Arc::new(Desync::new(()));
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

    ///
    /// Performs the specified function with this log publisher registered as the current publisher
    ///
    pub fn with<Res, ActionFn: FnOnce() -> Res>(&self, action: ActionFn) -> Res {
        // Remember the previous logger and set the current logger to this
        let previous_log = current_log();
        THREAD_LOGGER.with(|logger| *logger.borrow_mut() = Some(self.clone()));

        // Perform the action with the new current logger
        let result = action();

        // Reset the logger
        THREAD_LOGGER.with(|logger| *logger.borrow_mut() = Some(previous_log));

        // Return the result of the operation
        result
    }
}

///
/// Retrieves a reference to the log publisher for the current thread
///
pub (crate) fn current_log() -> LogPublisher {
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

///
/// Sends flo logs to a standard logger
///
pub fn send_logs_to(logger: Box<dyn Log+Send+'static>) {
    pipe_in(Arc::clone(&CORE_PROCESSOR), CORE_LOGGER.subscribe(), move |_, message| {
        // Generate metadata for the level/target
        let metadata = MetadataBuilder::new()
            .level(message.level().into())
            .target(message.field_value("target").unwrap_or("flo_logger"))
            .build();

        // Metadata for the log message itself (we don't actually format a whole lot of arguments)
        logger.log(&RecordBuilder::new()
            .metadata(metadata)
            .file(message.field_value("file"))
            .line(message.field_value("line").and_then(|line_str| line_str.parse::<u32>().ok()))
            .args(format_args!("{}", message.message()))
            .build());

        Box::pin(future::ready(()))
    })
}

struct FloLog;
static FLO_LOG: FloLog = FloLog;

impl Log for FloLog {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // No filtering for now
        true
    }

    fn log(&self, record: &Record) {
        // Get the current log
        let log = current_log();

        // Format into a log message
        let msg = format!("{}", record.args());
        let msg = vec![
            ("target", record.target()),
            ("message", &msg)
        ].into_iter().collect::<HashMap<_, _>>();
        let msg: (log::Level, _) = (record.level().into(), msg);

        // Send to the log
        log.log(msg);
    }

    fn flush(&self) {
        // Nothing to flush
    }
}

///
/// Creates a Rust logger that sends any log messages received for the main log crate to the current flo logger
/// for the thread
///
pub fn send_rust_logs_to_flo_logs() -> Result<(), SetLoggerError> {
    set_max_level(LevelFilter::Debug);
    set_logger(&FLO_LOG)
}

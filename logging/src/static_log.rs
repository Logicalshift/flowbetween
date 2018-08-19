use super::message::*;
use super::publisher::*;
use super::log_subscriber::*;

use log::*;
use desync::*;

use std::sync::*;
use std::cell::*;

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
pub fn send_logs_to(logger: Box<Log+Send+'static>) {
    pipe_in(Arc::clone(&CORE_PROCESSOR), CORE_LOGGER.subscribe(), move |_, message| {
        if let Ok(message) = message {
            let msg     = message.message().to_string();
            let args    = format_args!("How to fix lifetime?").to_owned();

            // Generate metadata for the level/target
            let metadata = MetadataBuilder::new()
                .level(message.level().into())
                .target(message.field_value("target").unwrap_or("flo_logger"))
                .build();
            
            // Metadata for the log message itself (we don't actually format a whole lot of arguments)
            let record = RecordBuilder::new()
                .metadata(metadata)
                .file(message.field_value("file"))
                .line(message.field_value("line").and_then(|line_str| line_str.parse::<u32>().ok()))
                .args(args)
                .build();
            
            // Send to the standard logger
            logger.log(&record);
        }
    })
}

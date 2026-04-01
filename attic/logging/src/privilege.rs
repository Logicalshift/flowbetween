///
/// The privilege level of a log message (an indication of who is allowed to read it)
///
/// This is mostly informational but potentially useful for generating logs that can be read by the user of
/// a multi-user application.
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LogPrivilege {
    /// Anyone can read this message
    All,

    /// Only the user and the application can read this message
    User,

    /// Only the application can read this message
    Application
}

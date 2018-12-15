///
/// Update latch commands
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum UpdateLatch {
    /// Stop processing updates
    Suspend,

    /// Start processing updates
    Resume
}
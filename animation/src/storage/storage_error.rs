///
/// Errors from the storage API
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StorageError {
    /// General failure
    General,

    /// The storage could not be initialised
    FailedToInitialise,

    /// The storage cannot continue because of an eariler error
    CannotContinueAfterError
}

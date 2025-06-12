use uuid::*;
use ::serde::*;

///
/// Identifier used to specify a dialog within the flowbetween app
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DialogId(Uuid);

impl DialogId {
    ///
    /// Creates a unique new dialog ID
    ///
    pub fn new() -> Self {
        DialogId(Uuid::new_v4())
    }
}

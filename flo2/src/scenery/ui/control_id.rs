use uuid::*;
use ::serde::*;

///
/// Identifier used to specify a control within the flowbetween app
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControlId(Uuid);

impl ControlId {
    ///
    /// Creates a unique new control ID
    ///
    pub fn new() -> Self {
        ControlId(Uuid::new_v4())
    }
}
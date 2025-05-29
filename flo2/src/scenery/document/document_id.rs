use uuid::*;
use ::serde::*;

///
/// Identifier used to specify a document within the flowbetween app
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(Uuid);

impl DocumentId {
    ///
    /// Creates a unique new document ID
    ///
    pub fn new() -> Self {
        DocumentId(Uuid::new_v4())
    }
}
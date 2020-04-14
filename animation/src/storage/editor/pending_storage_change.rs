use super::element_wrapper::*;
use super::super::storage_api::*;

use std::sync::*;
use std::collections::{HashMap};

#[derive(Clone)]
enum ElementChange {
    /// A change that is stored as an element wrapper
    Wrapper(Arc<ElementWrapper>),

    /// A change that has already been serialized
    Serialized(String)
}

///
/// Represents a storage change that is being built up
///
/// In order to reduce the number of writes to the underlying storage, we can perform all of the
/// element changes at the start, once per element. The actual contents of the element are not
/// read during the update so only the last update is required
///
#[derive(Clone)]
pub struct PendingStorageChange {
    /// The most recent version of the elements that these
    element_changes: HashMap<i64, ElementChange>,

    /// The other changes to perform
    storage_changes: Vec<StorageCommand>
}

impl PendingStorageChange {
    ///
    /// Creates a new, empty storage change
    ///
    pub fn new() -> PendingStorageChange {
        PendingStorageChange {
            element_changes:    HashMap::new(),
            storage_changes:    vec![]
        }
    }
}

impl IntoIterator for PendingStorageChange {
    type Item       = StorageCommand;
    type IntoIter   = Box<dyn Iterator<Item=StorageCommand>>;

    fn into_iter(self) -> Box<dyn Iterator<Item=StorageCommand>> {
        // First update the elements
        let update_elements = self.element_changes.into_iter()
            .map(|(element_id, change)| {
                match change {
                    ElementChange::Wrapper(wrapper) => StorageCommand::WriteElement(element_id, wrapper.serialize_to_string()),
                    ElementChange::Serialized(data) => StorageCommand::WriteElement(element_id, data)
                }
            });

        // Next, perform any other storage command
        let other_commands  = self.storage_changes.into_iter();

        // Chain together for the result
        // The element updates must come first so any new elements are generated but otherwise their ordering should not matter
        Box::new(update_elements.chain(other_commands))
    }
}

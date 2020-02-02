use super::storage_api::*;

use ::desync::*;

use futures::prelude::*;

use std::sync::*;

struct InMemoryStorageCore {
}

///
/// Provides an implementation of the storage API that stores its data in memory
///
pub struct InMemoryStorage {
    /// Where the data is stored for this object 
    storage: Arc<Desync<InMemoryStorageCore>>
}

impl InMemoryStorage {
    ///
    /// Creates a new in-memory storage for an animation
    ///
    pub fn new() -> InMemoryStorage {
        // Create the core
        let core = InMemoryStorageCore {
        };

        // And the storage
        InMemoryStorage {
            storage: Arc::new(Desync::new(core))
        }
    }

    ///
    /// Returns the responses for a stream of commands
    ///
    pub fn get_responses<CommandStream: 'static+Send+Unpin+Stream<Item=Vec<StorageCommand>>>(&self, commands: CommandStream) -> impl Send+Unpin+Stream<Item=Vec<StorageResponse>> {
        pipe(Arc::clone(&self.storage), commands, |storage, commands| {
            storage.run_commands(commands)
        })
    }
}

impl InMemoryStorageCore {
    ///
    /// Runs a series of storage commands on this store
    ///
    pub fn run_commands(&mut self, commands: Vec<StorageCommand>) -> Vec<StorageResponse> {
        // TODO
        vec![]
    }
}

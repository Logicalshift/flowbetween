use super::storage_api::*;

use ::desync::*;

use futures::prelude::*;

use std::sync::*;

struct InMemoryStorageCore {
    animation_properties: Option<String>
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
            animation_properties: None
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
        let mut response = vec![];

        for command in commands.into_iter() {
            use self::StorageCommand::*;

            match command {
                WriteAnimationProperties(props)                     => { self.animation_properties = Some(props); response.push(StorageResponse::Updated); }
                ReadAnimationProperties                             => { response.push(self.animation_properties.as_ref().map(|props| StorageResponse::AnimationProperties(props.clone())).unwrap_or(StorageResponse::NotFound)); }
                WriteEdit(edit)                                     => { }
                ReadHighestUnusedElementId                          => { }
                ReadEditLogLength                                   => { }
                ReadEdits(edit_range)                               => { }
                WriteElement(element_id, value)                     => { }
                ReadElement(element_id)                             => { }
                DeleteElement(element_id)                           => { }
                AttachElementToElement(target_id, attachment_id)    => { }
                DetachElementFromElement(target_id, attachment_id)  => { }
                AddLayer(layer_id)                                  => { }
                WriteLayerProperties(layer_id, properties)          => { }
                ReadLayerProperties(layer_id)                       => { }
                DeleteLayer(layer_id)                               => { }
                OrderLayer(layer_id, ordering)                      => { }
                AddKeyFrame(layer_id, when)                         => { }
                DeleteKeyFrame(layer_id, when)                      => { }
                AttachElementToLayer(layer_id, element_id, when)    => { }
                DetachElementFromLayer(element_id)                  => { }
                ReadElementsForKeyFrame(layer_id, when)             => { }
            }
        }

        response
    }
}

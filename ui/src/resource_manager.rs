use desync::*;

use std::ops::*;
use std::sync::*;
use std::collections::*;

///
/// Core data storage for the resource manager
/// 
struct ResourceManagerCore<T: Send+Sync> {
    /// Resources being managed by this object
    resources: Vec<WeakResource<T>>,

    /// Locations of free slots within the resource list
    free_slots: Vec<usize>,

    /// Next number of resources before we try cleaning
    clean_size: usize,

    /// Resources that have been assigned a name
    named_resources: HashMap<String, Resource<T>>
}

///
/// The resource manager is used to track resources of a particular type
/// 
pub struct ResourceManager<T: Send+Sync> {
    /// Core of the resource manager
    core: Desync<ResourceManagerCore<T>>,
}

///
/// Resource reference that will be released when done
///
struct WeakResource<T> {
    /// Weak reference to the resource
    resource: Weak<T>
}

///
/// Represents a resource being managed by the resource manager. Resources are removed
/// from the manager when all copies are dropped.
///
pub struct Resource<T> {
    /// Identifier for this resource
    id: u32,

    /// Data for the resource
    resource: Arc<T>
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Resource<T> {
        Resource { id: self.id, resource: self.resource.clone() }
    }
}

impl<T> Resource<T> {
    ///
    /// Retrieves the ID for this resource
    /// 
    pub fn id(&self) -> u32 { 
        self.id
    }
}

impl<T> Deref for Resource<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.resource
    }
}

impl<T: Send+Sync> ResourceManagerCore<T> {
    ///
    /// Registers a resource with this core
    /// 
    fn register(&mut self, resource: T) -> Resource<T> {
        // Pick an ID for the resource
        let id = self.free_slots.pop().unwrap_or(self.resources.len());
        
        // Create the resource object
        let resource = Resource {
            id:         id as u32,
            resource:   Arc::new(resource)
        };

        // Store it in the list of known resources
        let weak = WeakResource { resource: Arc::downgrade(&resource.resource) };
        if id >= self.resources.len() {
            self.resources.push(weak);
        } else {
            self.resources[id] = weak;
        }

        resource
    }

    ///
    /// Detects any resource slots that are free and marks them as ready
    /// 
    fn clean_resources(&mut self) {
        let mut new_free_slots = vec![];

        // Mark any resources that no longer exist as free
        for resource_id in 0..self.resources.len() {
            if self.resources[resource_id].resource.upgrade().is_none() {
                new_free_slots.push(resource_id);
            }
        }

        // Swap out the free slots
        self.free_slots = new_free_slots;

        // Clean again after adding 16 new resources
        self.clean_size = self.resources.len() - self.free_slots.len() + 16;
    }
}

impl<T: 'static+Send+Sync> ResourceManager<T> {
    ///
    /// Creates a new resource manager
    ///
    pub fn new() -> ResourceManager<T> {
        ResourceManager {
            core: Desync::new(ResourceManagerCore {
                resources:          vec![],
                named_resources:    HashMap::new(),
                free_slots:         vec![],
                clean_size:         1
            })
        }
    }

    ///
    /// Registers a resource with this object
    ///
    pub fn register(&self, data: T) -> Resource<T> {
        self.core.sync(move |core| {
            // Register with the core
            let resource = core.register(data);

            // Clean out expired resources the core has grown to a certain size
            if core.resources.len() >= core.clean_size {
                self.core.async(|core| core.clean_resources());
            }

            resource
        })
    }

    ///
    /// Given a resource ID, returns the corresponding resource
    ///
    pub fn get_resource_with_id(&self, id: u32) -> Option<Resource<T>> {
        self.core.sync(move |core| {
            if id as usize >= core.resources.len() {
                // Outside the bounds of the known resources
                None
            } else {
                // Try to upgrade the weak ref with this ID
                let weak = &core.resources[id as usize];
                if let Some(data_ref) = weak.resource.upgrade() {
                    // Generate a Resource<T> fro this
                    Some(Resource { id: id, resource: data_ref })
                } else {
                    // Resource is not in this manager
                    None
                }
            }
        })
    }

    ///
    /// Assigns a name to a particular resource
    ///
    pub fn assign_name(&self, resource: &Resource<T>, name: &str) {
        // Clone the resource to get the version we should name
        let to_name     = resource.clone();
        let name_string = String::from(name);

        // Store the name in the core
        self.core.async(move |core| {
            core.named_resources.insert(name_string, to_name);
        });
    }

    ///
    /// Retrieves a resource by name
    ///
    pub fn get_named_resource(&self, name: &str) -> Option<Resource<T>> {
        self.core.sync(move |core| {
            core.named_resources
                .get(&String::from(name))
                .map(|res| res.clone())
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;

    #[test]
    fn can_create_resource() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);

        assert!(*resource == 2);
    }

    #[test]
    fn can_retrieve_resource_by_id() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);

        let id = resource.id();

        assert!(resource_manager.get_resource_with_id(id).map(|x| *x) == Some(2));
    }

    #[test]
    fn bad_ids_dont_exist() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);

        let id = resource.id()+1;

        assert!(resource_manager.get_resource_with_id(id).map(|x| *x) == None);
    }

    #[test]
    fn resources_expire_when_released() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);
        let id = resource.id();

        mem::drop(resource);

        assert!(resource_manager.get_resource_with_id(id).map(|x| *x) == None);
    }

    #[test]
    fn can_retrieve_resource_by_name() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);

        resource_manager.assign_name(&resource, "Mr Resource");

        assert!(resource_manager.get_named_resource("Mr Resource").map(|x| *x) == Some(2));
    }

    #[test]
    fn named_resources_do_not_expire() {
        let resource_manager = ResourceManager::new();

        let resource = resource_manager.register(2);
        resource_manager.assign_name(&resource, "Mr Resource");
        mem::drop(resource);

        assert!(resource_manager.get_named_resource("Mr Resource").map(|x| *x) == Some(2));
    }

    #[test]
    fn can_manage_many_resources() {
        let resource_manager = ResourceManager::new();

        let mut resource = vec![];

        for i in 0..100 {
            resource.push(resource_manager.register(i));
        }

        for i in 0..100 {
            assert!(resource_manager.get_resource_with_id(resource[i].id()).map(|x| *x) == Some(i));
        }
    }
}

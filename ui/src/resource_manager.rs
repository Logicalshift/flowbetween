use ::desync::*;

use std::fmt;
use std::ops::*;
use std::sync::*;
use std::fmt::Debug;
use std::collections::*;

///
/// Core data storage for the resource manager
///
struct ResourceManagerCore<T: Send+Sync> {
    /// Resources being managed by this object
    resources: Vec<WeakResource<T>>,

    /// Stores the name for a particular ID
    name_for_id: Vec<Arc<Mutex<Option<String>>>>,

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

    /// The name of this resource, if it has one
    name: Arc<Mutex<Option<String>>>,

    /// Data for the resource
    resource: Arc<T>
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Resource<T> {
        Resource {
            id:         self.id,
            resource:   self.resource.clone(),
            name:       self.name.clone()
        }
    }
}

impl<T> Resource<T> {
    ///
    /// Retrieves the ID for this resource
    ///
    pub fn id(&self) -> u32 {
        self.id
    }

    ///
    /// Retrieves the name for this resource, if it has one.
    ///
    /// Note that if another resource has been assigned the same name, this might return
    /// the old name for a while (use the version in the resource manager if this matters)
    ///
    pub fn name(&self) -> Option<String> {
        let name = self.name.lock().unwrap();

        if let Some(ref name) = *name {
            Some(name.clone())
        } else {
            None
        }
    }

    ///
    /// Retrieves an Arc reference of the resource
    ///
    pub fn get_arc(&self) -> Arc<T> {
        self.resource.clone()
    }
}

impl<T> Deref for Resource<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.resource
    }
}

impl<T> PartialEq for Resource<T> {
    fn eq(&self, other: &Resource<T>) -> bool {
        other.id == self.id
    }
}

impl<T> Debug for Resource<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let name = self.name.lock().unwrap().clone();

        if let Some(name) = name {
            fmt.write_fmt(format_args!("{} \"{}\"", self.id, name))
        } else {
            fmt.write_fmt(format_args!("{}", self.id))
        }
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
            resource:   Arc::new(resource),
            name:       self.set_name_for_id(id as u32, None)
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
    /// Updates the name for a particular ID
    ///
    fn set_name_for_id(&mut self, id: u32, name: Option<String>) -> Arc<Mutex<Option<String>>> {
        // Extend so we have enough slots in name_for_id
        while self.name_for_id.len() <= id as usize {
            self.name_for_id.push(Arc::new(Mutex::new(None)))
        }

        // Set the value in the specified slot to the value that was requested
        let name_slot = &self.name_for_id[id as usize];
        *name_slot.lock().unwrap() = name;
        name_slot.clone()
    }

    ///
    /// Retrieves the name for a particular ID
    ///
    fn get_name_for_id(&mut self, id: u32) -> Arc<Mutex<Option<String>>> {
        if (id as usize) < self.name_for_id.len() as usize {
            self.name_for_id[id as usize].clone()
        } else {
            self.set_name_for_id(id, None)
        }
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
                name_for_id:        vec![],
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
                self.core.desync(|core| core.clean_resources());
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
                let name = core.get_name_for_id(id);
                let weak = &core.resources[id as usize];
                if let Some(data_ref) = weak.resource.upgrade() {
                    // Generate a Resource<T> fro this
                    Some(Resource {
                        id:         id,
                        resource:   data_ref,
                        name:       name
                    })
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
        self.core.desync(move |core| {
            // Remove the name from the previous owner
            {
                let previous_owner = core.named_resources.get(&name_string);
                if let Some(previous_owner) = previous_owner {
                    // We don't remove the name if we've re-assigned it to the same object
                    if previous_owner.id != to_name.id {
                        // Was owned by someone else...
                        let mut previous_name = previous_owner.name.lock().unwrap();

                        if *previous_name == Some(name_string.clone()) {
                            // Still has its old name as the 'official' one
                            *previous_name = None;
                        }
                    }
                }
            }

            // Store as the resource with this name
            core.named_resources.insert(name_string, to_name);
        });

        // Store the name in the resource
        *resource.name.lock().unwrap() = Some(String::from(name));
    }

    ///
    /// Retrieves the name for a particular resource
    ///
    pub fn get_name(&self, resource: &Resource<T>) -> Option<String> {
        self.core.sync(move |_core| {
            resource.name()
        })
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
    fn resources_start_with_no_name() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);

        assert!(resource.name() == None);
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
    fn can_replace_named_resources() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);
        let resource2           = resource_manager.register(3);

        resource_manager.assign_name(&resource, "Mr Resource");
        resource_manager.assign_name(&resource2, "Mr Resource");

        assert!(resource_manager.get_named_resource("Mr Resource").map(|x| *x) == Some(3));
    }

    #[test]
    fn can_get_name_from_resource() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);

        resource_manager.assign_name(&resource, "Mr Resource");

        assert!(resource.name() == Some(String::from("Mr Resource")));
    }

    #[test]
    fn can_get_name_after_retrieving_by_id() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);

        resource_manager.assign_name(&resource, "Mr Resource");

        let id = resource.id();
        assert!(resource_manager.get_resource_with_id(id).map(|x| x.name()).unwrap() == Some(String::from("Mr Resource")));
    }

    #[test]
    fn name_expires_when_replaced() {
        let resource_manager    = ResourceManager::new();
        let resource            = resource_manager.register(2);
        let resource2           = resource_manager.register(3);

        resource_manager.assign_name(&resource, "Mr Resource");
        resource_manager.assign_name(&resource2, "Mr Resource");

        // The resource_manager.get_name() synchronises so removes the possibility of the old resource retaining its name for a while
        assert!(resource_manager.get_name(&resource) == None);
        assert!(resource_manager.get_name(&resource2) == Some(String::from("Mr Resource")));
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

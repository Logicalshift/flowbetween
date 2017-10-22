use super::controller::*;

use std::sync::*;

///
/// Tracks differences in the viewmodel attached to a controller and its subtree
/// 
pub struct DiffViewModel {
    /// The controller that owns the viewmodel (if it's still live)
    controller: Weak<Controller>,
}

///
/// Watches for changes in a viewmodel
///
pub struct WatchViewModel {
    /// The controller to watch
    controller: Weak<Controller>
}

impl DiffViewModel {
    ///
    /// Creates a new viewmodel tracker for a particular controller
    ///
    pub fn new(controller: Arc<Controller>) -> DiffViewModel {
        DiffViewModel { controller: Arc::downgrade(&controller) }
    }

    ///
    /// Reads the current state of the controller and creates a watcher for any changes that
    /// might occur to it.
    ///
    pub fn watch(&self) -> WatchViewModel {
        WatchViewModel { controller: self.controller.clone() }
    }
}
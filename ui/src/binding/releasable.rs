use super::traits::*;

use std::sync::*;

///
/// A notifiable that can be released (and then tidied up later)
///
#[derive(Clone)]
pub struct ReleasableNotifiable {
    target: Arc<Mutex<Option<Arc<Notifiable>>>>
}

impl ReleasableNotifiable {
    ///
    /// Creates a new releasable notifiable object
    ///
    pub fn new(target: Arc<Notifiable>) -> ReleasableNotifiable {
        ReleasableNotifiable {
            target: Arc::new(Mutex::new(Some(target)))
        }
    }

    ///
    /// Marks this as changed and returns whether or not the notification was called
    ///
    pub fn mark_as_changed(&self) -> bool {
        // Get a reference to the target via the lock
        let target = {
            // Reset the optional item so that it's 'None'
            let target = self.target.lock().unwrap();

            // Send to the target
            target.clone()
        };

        // Send to the target
        if let Some(ref target) = target {
            target.mark_as_changed();
            true
        } else {
            false
        }
    }

    ///
    /// True if this item is still in use
    ///
    pub fn is_in_use(&self) -> bool {
        self.target.lock().unwrap().is_some()
    }
}

impl Releasable for ReleasableNotifiable {
    fn done(&mut self) {
        // Reset the optional item so that it's 'None'
        let mut target = self.target.lock().unwrap();

        *target = None;
    }
}

impl Notifiable for ReleasableNotifiable {
    fn mark_as_changed(&self) {
        // Get a reference to the target via the lock
        let target = {
            // Reset the optional item so that it's 'None'
            let target = self.target.lock().unwrap();

            // Send to the target
            target.clone()
        };

        // Make sure we're calling out to mark_as_changed outside of the lock
        if let Some(target) = target {
            target.mark_as_changed();
        }
    }
}

impl Releasable for Vec<Box<Releasable>> {
    fn done(&mut self) {
        for item in self.iter_mut() {
            item.done();
        }
    }
}

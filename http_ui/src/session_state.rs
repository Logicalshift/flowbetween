use ui::*;

use uuid::*;
use std::sync::*;

///
/// The session state object represents the stored state of a particular session
///
pub struct SessionState {
    /// A string identifying this session
    session_id: String,

    // TODO: move this stuff into a 'core' struct protected by a mutex instead of having a bunch of mutexes

    /// The UI tree for this session
    ui_tree: Mutex<Box<Bound<Control>>>,

    /// Lifetime that tracks if we're watching the tree
    ui_tree_watcher_lifetime: Mutex<Box<Releasable>>,

    /// Binding that tracks if the tree has changed since the last time it was diffed
    tree_has_changed: Binding<bool>
}

impl SessionState {
    ///
    /// Creates a new session state
    ///
    pub fn new() -> SessionState {
        let session_id                      = Uuid::new_v4().simple().to_string();
        let mut tree: Box<Bound<Control>>   = Box::new(bind(Control::container()));
        let has_changed                     = bind(false);
        let watcher_lifetime                = Self::watch_tree(&mut tree, &has_changed);

        SessionState { 
            session_id:                 session_id,
            ui_tree:                    Mutex::new(tree),
            ui_tree_watcher_lifetime:   Mutex::new(watcher_lifetime),
            tree_has_changed:           has_changed
        }
    }

    ///
    /// Sets has_changed to true when the ui_tree changes
    ///
    fn watch_tree(ui_tree: &mut Box<Bound<Control>>, has_changed: &Binding<bool>) -> Box<Releasable> {
        let mut changed = has_changed.clone();
        ui_tree.when_changed(notify(move || changed.set(true)))
    }

    ///
    /// Retrieves the ID of this session
    ///
    pub fn id(&self) -> String {
        self.session_id.clone()
    }

    ///
    /// Replaces the UI tree in this session
    ///
    pub fn set_ui_tree<TBinding: 'static+Bound<Control>>(&self, new_tree: TBinding) {
        // TODO: this would be way easier to follow if ther were a core class protected by a mutex
        self.ui_tree_watcher_lifetime.lock().unwrap().done();
        *self.ui_tree.lock().unwrap() = Box::new(new_tree);

        *self.ui_tree_watcher_lifetime.lock().unwrap() = Self::watch_tree(&mut *self.ui_tree.lock().unwrap(), &self.tree_has_changed);
    }

    ///
    /// Retrieves the current state of the UI for this session
    ///
    pub fn entire_ui_tree(&self) -> Control {
        self.ui_tree.lock().unwrap().get()
    }
}

impl Drop for SessionState {
    fn drop(&mut self) {
        self.ui_tree_watcher_lifetime.lock().unwrap().done();
    }
}

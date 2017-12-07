use super::canvas_state::*;
use super::canvas_update::*;

use binding::*;
use ui::*;

use uuid::*;
use std::sync::*;
use std::mem;

///
/// The core of the session state
///
struct SessionStateCore {
    /// The tree attached to this state
    ui_tree: Box<Bound<Control>>,

    /// The previous state of the tree
    previous_tree: Binding<Option<Control>>,

    /// Lifetime of the watcher that
    ui_tree_watcher_lifetime: Box<Releasable>,

    /// Binding that specifies whether or not the tree has changed
    tree_has_changed: Binding<bool>,

    /// Tracks the differences for the current view model
    viewmodel_diff: Option<(DiffViewModel, WatchViewModel)>,

    /// Tracks the state of the canvases used by this object
    canvas_state: CanvasState
}

///
/// The session state object represents the stored state of a particular session
///
pub struct SessionState {
    /// A string identifying this session
    session_id: String,

    /// The core of the state
    core: Mutex<SessionStateCore>
}

impl SessionState {
    ///
    /// Creates a new session state
    ///
    pub fn new() -> SessionState {
        let session_id                      = Uuid::new_v4().simple().to_string();
        let mut tree: Box<Bound<Control>>   = Box::new(bind(Control::container()));
        let has_changed                     = bind(false);
        let watcher_lifetime                = Self::watch_tree(&mut tree, has_changed.clone());
        let canvas_state                    = CanvasState::new(&tree);

        let core = SessionStateCore {
            ui_tree:                    tree,
            previous_tree:              bind(None),
            ui_tree_watcher_lifetime:   watcher_lifetime,
            tree_has_changed:           has_changed,
            viewmodel_diff:             None,
            canvas_state:               canvas_state
        };

        SessionState { 
            session_id: session_id,
            core:       Mutex::new(core)
        }
    }

    ///
    /// Sets has_changed to true when the ui_tree changes
    ///
    fn watch_tree(ui_tree: &mut Box<Bound<Control>>, mut has_changed: Binding<bool>) -> Box<Releasable> {
        ui_tree.when_changed(notify(move || has_changed.set(true)))
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
    pub fn set_ui_tree(&self, new_tree: Box<Bound<Control>>) {
        let mut core = self.core.lock().unwrap();

        // Stop watching the old tree
        core.ui_tree_watcher_lifetime.done();

        // Store the new UI tree in this object
        core.ui_tree = new_tree;

        // Whenever the new tree is changed, set the has_changed binding
        let mut has_changed = core.tree_has_changed.clone();
        has_changed.set(true);
        core.ui_tree_watcher_lifetime = Self::watch_tree(&mut core.ui_tree, has_changed);

        // Watch for canvas changes
        core.canvas_state = CanvasState::new(&core.ui_tree);
    }

    ///
    /// Retrieves the current state of the UI for this session
    ///
    pub fn entire_ui_tree(&self) -> Control {
        let core = self.core.lock().unwrap();

        core.ui_tree.get()
    }

    ///
    /// Returns the differences between the specified tree and the
    /// active UI tree
    ///
    pub fn ui_differences(&self, compare_to: &Control) -> Vec<Diff<Control>> {
        // Get the current state of the UI tree
        let core            = self.core.lock().unwrap();
        let current_tree    = core.ui_tree.get();

        // Compare to the supplied tree
        diff_tree(compare_to, &current_tree)
    }

    ///
    /// Begins watching a particular controller's viewmodel for changes 
    ///
    pub fn watch_controller_viewmodel(&self, controller: Arc<Controller>) {
        let new_diff        = DiffViewModel::new(controller);
        let new_watcher     = new_diff.watch();
        let mut core        = self.core.lock().unwrap();

        core.viewmodel_diff = Some((new_diff, new_watcher));
    }

    ///
    /// Cycles the current viewmodel watch and returns the updates to perform
    ///
    pub fn cycle_viewmodel_watch(&self) -> Vec<ViewModelUpdate> {
        let mut core        = self.core.lock().unwrap();
        let mut old_state   = None;

        // Swap out the old state so we can cycle it
        mem::swap(&mut old_state, &mut core.viewmodel_diff);

        if let Some((diff, watcher)) = old_state {
            // Cycle the watcher if we found one in the core
            let (result, new_watcher) = diff.rotate_watch(watcher);
            core.viewmodel_diff = Some((diff, new_watcher));

            result
        } else {
            // No controller is being watched, so there's nothing to cycle
            vec![]
        }
    }

    ///
    /// Retrieves the canvas updates since the last time this was called 
    ///
    pub fn latest_canvas_updates(&self) -> Vec<CanvasUpdate> {
        self.core.lock().unwrap().canvas_state.latest_updates()
    }
}

impl Drop for SessionState {
    fn drop(&mut self) {
        let mut core = self.core.lock().unwrap();

        core.ui_tree_watcher_lifetime.done();
    }
}

use super::update::*;
use super::super::control::*;

use canvas::*;
use binding::*;

use desync::*;

use futures::*;
use futures::executor;
use futures::executor::{Spawn, Notify};

use std::sync::*;
use std::collections::{HashMap, HashSet};

///
/// The canvas state is used to monitor updates to canvases stored in a control hierarchy
///
pub struct CanvasState {
    /// Stores information about the canvases used by this item
    core: Arc<Desync<CanvasStateCore>>,

    /// Releasable that monitors the lifetime of the control watcher
    control_watch_lifetime: Box<Releasable>
}

///
/// Path to a canvas in this state object
///
#[derive(Clone, PartialEq, Eq, Hash)]
struct CanvasPath {
    /// The path to the controller that owns this canvas
    controller_path: Vec<String>,

    /// The name of the canvas within the controller
    canvas_name: String,
}

///
/// Data stored about a canvas that we're tracking
///
struct CanvasTracker {
    /// Stream for this canvas
    command_stream: Spawn<Box<Stream<Item=Draw,Error=()>+Send>>
}

///
/// Core state information for the canvas state object
///
struct CanvasStateCore {
    /// The controls that we're tracking
    root_control: BindRef<Control>,

    /// Set to true if the controls in this canvas have changed since they were last updated
    controls_updated: bool,

    /// The canvases that are being tracked by this state object
    canvases: HashMap<CanvasPath, CanvasTracker>
}

struct NotifyNothing;
impl Notify for NotifyNothing {
    fn notify(&self, _: usize) { }
}

impl CanvasStateCore {
    ///
    /// Find the canvases in the controls attached to this canvas and add any new
    /// ones to the ones being tracked, and removes any canvases that are in the
    /// list but which are not present in the control any more.
    ///
    fn update_canvases(&mut self) {
        let control             = self.root_control.get();
        let mut found_canvases  = HashSet::new();

        // Update the list of canvases found in the control
        self.update_control(&control, &vec![], &mut found_canvases);

        // Find the canvases that are missing from this control
        let missing_canvases: Vec<CanvasPath> = self.canvases.keys()
            .filter(|canvas_path| !found_canvases.contains(canvas_path))
            .map(|missing_path| missing_path.clone())
            .collect();

        // Remove all of the missing canvases
        for canvas_path in missing_canvases.into_iter() {
            self.canvases.remove(&canvas_path);
        }
    }

    ///
    /// Redraws any canvases attached to a particular control
    /// 
    fn redraw_control(&mut self, control: &Control) {
        // If the control contains a canvas...
        if let Some(canvas) = control.canvas_resource() {
            // Perform any redrawing action that the control might require
            canvas.redraw_if_invalid();
        }

        // Recurse into subcomponents
        if let Some(subcomponents) = control.subcomponents() {
            for subcomponent in subcomponents {
                self.redraw_control(subcomponent);
            }
        }
    }

    ///
    /// Updates the list of canvases in this control
    ///
    fn update_control(&mut self, control: &Control, controller_path: &Vec<String>, found_canvases: &mut HashSet<CanvasPath>) {
        // If this control has an attached canvas, then watch it if it's not already being watched
        if let Some(canvas) = control.canvas_resource() {
            // Make the canvas name...
            let canvas_name = if let Some(name) = canvas.name() {
                String::from(name)
            } else {
                canvas.id().to_string()
            };

            // ...and the path
            let path = CanvasPath { controller_path: controller_path.clone(), canvas_name: canvas_name };

            // Create a new tracker if this canvas is not already being watched
            if !self.canvases.contains_key(&path) {
                let stream  = executor::spawn((*canvas).stream());
                let tracker = CanvasTracker { command_stream: stream };

                self.canvases.insert(path.clone(), tracker);
            }

            // Mark this as a 'found canvas' so it's not removed from this object
            found_canvases.insert(path);
        }

        // Extend the controller path if this control has a controller
        let mut our_controller_path;
        let mut next_controller_path = controller_path;

        if let Some(controller) = control.controller() {
            our_controller_path = controller_path.clone();
            our_controller_path.push(String::from(controller));

            next_controller_path = &our_controller_path;
        }

        // Recurse into the subcomponents
        if let Some(subcomponents) = control.subcomponents() {
            for subcomponent in subcomponents {
                self.update_control(subcomponent, next_controller_path, found_canvases);
            }
        }
    }

    ///
    /// Polls for updates available in a particular canvas tracker
    /// 
    fn updates_for(tracker: &mut CanvasTracker) -> Vec<Draw> {
        let mut result = vec![];

        while let Ok(Async::Ready(Some(command))) = tracker.command_stream.poll_stream_notify(&Arc::new(NotifyNothing), 0) {
            result.push(command);
        }

        result
    }

    ///
    /// Polls the canvases in this object for their latest updates
    /// 
    fn latest_updates(&mut self) -> Vec<CanvasDiff> {
        // Get the updates for all of the canvases
        let mut updates = vec![];

        for (path, mut canvas) in self.canvases.iter_mut() {
            let canvas_updates = Self::updates_for(&mut canvas);

            if canvas_updates.len() > 0 {
                // If this canvas has changed, encode its updates
                updates.push(CanvasDiff {
                    controller:     path.controller_path.clone(),
                    canvas_name:    path.canvas_name.clone(),
                    updates:        canvas_updates
                });
            }
        }

        updates
    }
}

impl CanvasState {
    ///
    /// Creates a new canvas state
    /// 
    pub fn new(control: &BindRef<Control>) -> CanvasState {
        // Clone the control so we can watch it ourselves
        let control = control.clone();

        // Create the core
        let core = Arc::new(Desync::new(CanvasStateCore {
            root_control:       control,
            controls_updated:   true,
            canvases:           HashMap::new()
        }));

        // Mark the core as changed whenever the controls change
        let watch_core              = core.clone();
        let control_watch_lifetime  = core.sync(move |core| {
            core.root_control.when_changed(notify(move || watch_core.async(|core| core.controls_updated = true)))
        });

        // State is just a wrapper for the core
        CanvasState {
            core:                   core,
            control_watch_lifetime: control_watch_lifetime
        }
    }

    ///
    /// Finds the latest updates for this canvas
    ///
    pub fn latest_updates(&self) -> Vec<CanvasDiff> {
        // First redraw any canvases that might be in the control tree and out of date
        self.core.sync(|core| {
            let root_control = core.root_control.get();
            core.redraw_control(&root_control);
        });

        // Then find the latest updates
        self.core.sync(|core| {
            // Update the set of canvases that need to be checked
            if core.controls_updated {
                core.controls_updated = false;
                core.update_canvases();
            }

            // Return any updates we can find
            core.latest_updates()
        })
    }
}

impl Drop for CanvasState {
    fn drop(&mut self) {
        self.control_watch_lifetime.done();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::binding_canvas::*;
    use super::super::super::resource_manager::*;

    #[test]
    fn can_create_canvas_state()  {
        let resource_manager    = ResourceManager::new();
        let canvas              = resource_manager.register(BindingCanvas::new());
        let control: BindRef<Control> = BindRef::from(bind(Control::canvas().with(canvas)));

        let canvas_state        = CanvasState::new(&control);

        assert!(canvas_state.latest_updates().len() == 1);
        assert!(canvas_state.latest_updates().len() == 0);
    }

    #[test]
    fn canvas_updates_when_control_changes()  {
        let resource_manager    = ResourceManager::new();
        let canvas              = resource_manager.register(BindingCanvas::new());
        let mut control         = Binding::new(Control::canvas().with(canvas));

        let box_control: BindRef<Control> = BindRef::new(&control);
        let canvas_state        = CanvasState::new(&box_control);

        assert!(canvas_state.latest_updates().len() == 1);
        assert!(canvas_state.latest_updates().len() == 0);

        let canvas2 = resource_manager.register(BindingCanvas::new());
        control.set(Control::canvas().with(canvas2));

        assert!(canvas_state.latest_updates().len() == 1);
        assert!(canvas_state.latest_updates().len() == 0);
    }

    // TODO: do we have issues if the resource manager re-uses an ID?
    // TODO: test encoding of canvas path, canvas name
}

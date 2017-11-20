use ui::*;
use ui::canvas::*;
use binding::*;

use desync::*;

use futures::*;
use futures::executor;
use futures::executor::Spawn;

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
    root_control: Box<Bound<Control>>,

    /// Set to true if the controls in this canvas have changed since they were last updated
    controls_updated: bool,

    /// The canvases that are being tracked by this state object
    canvases: HashMap<CanvasPath, CanvasTracker>
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

                found_canvases.insert(path.clone());
                self.canvases.insert(path, tracker);
            }
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
}

impl CanvasState {
    ///
    /// Creates a new canvas state
    /// 
    pub fn new(control: &Box<Bound<Control>>) -> CanvasState {
        // Clone the control so we can watch it ourselves
        let control = control.clone_box();

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
}

use ui::*;
use ui::canvas::*;
use binding::*;

use desync::*;

use futures::*;
use futures::executor::*;

use std::sync::*;
use std::collections::HashMap;

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

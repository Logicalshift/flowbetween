use ui::*;
use ui::canvas::*;
use binding::*;

use desync::*;

use futures::*;
use futures::executor::*;

use std::collections::HashMap;

///
/// The canvas state is used to monitor updates to canvases stored in a control hierarchy
///
pub struct CanvasState {
    /// Stores information about the canvases used by this item
    core: Desync<CanvasStateCore>
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
    controls: Box<Bound<Control>>,

    /// Set to true if the controls in this canvas have changed since they were last updated
    controls_updated: bool,

    /// The canvases that are being tracked by this state object
    canvases: HashMap<CanvasPath, CanvasTracker>
}

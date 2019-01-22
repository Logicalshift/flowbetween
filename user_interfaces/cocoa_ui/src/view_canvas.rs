use super::canvas_state::*;
use super::canvas_context::*;
use super::core_graphics_ffi::*;

use flo_canvas::*;

///
/// Stores the state associated with a canvas specified for a view
///
pub struct ViewCanvas {
    /// A copy of the canvas for this view (used when we need to redraw the viewport)
    canvas: Canvas,

    /// The current state of the view canvas
    state: Option<CanvasState>
}

impl ViewCanvas {
    ///
    /// Creates a new canvas for a view
    ///
    pub fn new() -> ViewCanvas {
        ViewCanvas {
            canvas: Canvas::new(),
            state:  None
        }
    }

    ///
    /// Draws some actions to this view canvas
    ///
    pub fn draw(&mut self, actions: Vec<Draw>, context: CFRef<CGContextRef>) {
        // Write the actions to the canvas so we can redraw them later on
        self.canvas.write(actions.clone());

        // Update the existing canvas context
        unsafe {
            // Create the drawing context
            let mut context = CanvasContext::new(context, (0.0, 0.0), (1920.0, 1080.0), (1920.0, 1080.0));

            // Update the context state
            if let Some(state) = self.state.take() {
                context.set_state(state);
            }

            // Send the drawing commands
            actions.into_iter().for_each(|action| context.draw(&action));

            // Finished with the context, store the final state
            self.state = Some(context.to_state());
        }
    }
}

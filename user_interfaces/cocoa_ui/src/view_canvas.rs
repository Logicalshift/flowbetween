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

    /// The current size of the canvas
    size: CGSize,

    /// The visible area of the canvas (0,0 in core graphics terms is considered to be at the origin of this point)
    visible: CGRect,

    /// The current state of the view canvas
    state: Option<CanvasState>
}

impl ViewCanvas {
    ///
    /// Creates a new canvas for a view
    ///
    pub fn new() -> ViewCanvas {
        ViewCanvas {
            canvas:     Canvas::new(),
            size:       CGSize { width: 1.0, height: 1.0 },
            visible:    CGRect { origin: CGPoint { x: 0.0, y: 0.0 }, size: CGSize { width: 1.0, height: 1.0 } },
            state:      None
        }
    }

    ///
    /// Updates the size of the canvas
    ///
    pub fn set_viewport(&mut self, size: CGSize, visible: CGRect) {
        self.size       = size;
        self.visible    = visible;
    }

    ///
    /// Redraws the entire canvas
    ///
    pub fn redraw<ContextForLayer: FnMut(u32) -> (Option<CFRef<CGContextRef>>)>(&mut self, context_for_layer: ContextForLayer) {
        // Get the initial context
        let mut context_for_layer   = context_for_layer;
        let context                 = context_for_layer(0);

        if context.is_none() {
            // Nothing to do if we can't get the context for layer 0
            return;
        }
        let context = context.unwrap();

        // Fetch the current set of drawing instructions
        let actions = self.canvas.get_drawing();

        unsafe {
            // Reset the state
            let srgb    = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
            let state   = CanvasState::new(CFRef::from(srgb));

            self.state = Some(state);

            // Create the drawing context
            let viewport_origin = (self.visible.origin.x as f64, self.visible.origin.y as f64);
            let viewport_size   = (self.visible.size.width as f64, self.visible.size.height as f64);
            let canvas_size     = (self.size.width as f64, self.size.height as f64);

            let mut context = CanvasContext::new(context, viewport_origin, viewport_size, canvas_size);

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

    ///
    /// Draws some actions to this view canvas
    ///
    pub fn draw<ContextForLayer: FnMut(u32) -> (Option<CFRef<CGContextRef>>)>(&mut self, actions: Vec<Draw>, context_for_layer: ContextForLayer) {
        // Write the actions to the canvas so we can redraw them later on
        self.canvas.write(actions.clone());

        // Get the initial context
        let mut context_for_layer   = context_for_layer;
        let context                 = context_for_layer(0);

        if context.is_none() {
            // Nothing to do if we can't get the context for layer 0
            return;
        }
        let context = context.unwrap();

        // Update the existing canvas context
        unsafe {
            // Create the drawing context
            let viewport_origin = (self.visible.origin.x as f64, self.visible.origin.y as f64);
            let viewport_size   = (self.visible.size.width as f64, self.visible.size.height as f64);
            let canvas_size     = (self.size.width as f64, self.size.height as f64);

            let mut context = CanvasContext::new(context, viewport_origin, viewport_size, canvas_size);

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

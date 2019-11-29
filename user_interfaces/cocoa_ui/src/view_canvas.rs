use super::canvas_state::*;
use super::canvas_context::*;
use super::core_graphics_ffi::*;

use flo_canvas::*;

use objc::rc::*;

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
    state: Option<CanvasState>,

    /// Callback function that clears the canvas
    clear_canvas: Box<dyn FnMut() -> ()>,

    /// Callback function to create a copy of the layer with the specified ID
    copy_layer: Box<dyn FnMut(u32) -> StrongPtr>,

    /// Callback function to update the contents of a cached layer 
    update_layer: Box<dyn FnMut(u32, StrongPtr) -> ()>,

    /// Callback function to restore the state of a layer from a copy created previously with copy_layer
    restore_layer: Box<dyn FnMut(u32, StrongPtr) -> ()>
}

impl ViewCanvas {
    ///
    /// Creates a new canvas for a view
    ///
    pub fn new<ClearCanvasFn, CopyLayerFn, UpdateLayerFn, RestoreLayerFn>(clear_canvas: ClearCanvasFn, copy_layer: CopyLayerFn, update_layer: UpdateLayerFn, restore_layer: RestoreLayerFn) -> ViewCanvas
    where   ClearCanvasFn:  'static+FnMut() -> (),
            CopyLayerFn:    'static+FnMut(u32) -> StrongPtr,
            UpdateLayerFn:  'static+FnMut(u32, StrongPtr) -> (),
            RestoreLayerFn: 'static+FnMut(u32, StrongPtr) -> () {
        ViewCanvas {
            canvas:         Canvas::new(),
            size:           CGSize { width: 1.0, height: 1.0 },
            visible:        CGRect { origin: CGPoint { x: 0.0, y: 0.0 }, size: CGSize { width: 1.0, height: 1.0 } },
            state:          None,
            clear_canvas:   Box::new(clear_canvas),
            copy_layer:     Box::new(copy_layer),
            update_layer:   Box::new(update_layer),
            restore_layer:  Box::new(restore_layer)
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
    /// Retrieves the transform for this view canvsa
    ///
    pub fn get_transform(&self) -> CGAffineTransform {
        if let Some(state) = self.state.as_ref() {
            state.current_transform()
        } else {
            unsafe { CGAffineTransformIdentity }
        }
    }

    ///
    /// Performs a series of drawing actions on a graphics context
    ///
    fn perform_drawing_on_context<ContextForLayer: FnMut(u32) -> (Option<CFRef<CGContextRef>>), ActionIter: IntoIterator<Item=Draw>>(&mut self, actions: ActionIter, context_for_layer: ContextForLayer) {
        // Get the initial context
        let mut context_for_layer   = context_for_layer;
        let layer_context           = context_for_layer(0);

        if layer_context.is_none() {
            // Nothing to do if we can't get the context for layer 0
            return;
        }
        let layer_context = layer_context.unwrap();

        // Create the drawing context
        let viewport_origin = (self.visible.origin.x as f64, self.visible.origin.y as f64);
        let viewport_size   = (self.visible.size.width as f64, self.visible.size.height as f64);
        let canvas_size     = (self.size.width as f64, self.size.height as f64);

        let mut context = unsafe { CanvasContext::new(layer_context, viewport_origin, viewport_size, canvas_size) };

        // Update the context state
        if let Some(state) = self.state.take() {
            // Update the context to use the layer specified in the state
            let layer_context = context_for_layer(state.layer_id());
            if let Some(layer_context) = layer_context {
                // The canvas context doesn't deactivate itself on drop, so force it to deactivate by going through to_state
                context.to_state();
                context = unsafe { CanvasContext::new(layer_context, viewport_origin, viewport_size, canvas_size) };
            }

            // Set the initial state of the context
            context.set_state(state);
        }

        // Send the drawing commands
        for action in actions.into_iter() {
            use self::Draw::*;

            match action {
                ClearCanvas => {
                    unsafe {
                        // Invalidate the context and clear
                        context.to_state();
                        (self.clear_canvas)();

                        // Reset to layer 0
                        let layer_context = context_for_layer(0);
                        if let Some(layer_context) = layer_context {
                            // The canvas context doesn't deactivate itself on drop, so force it to deactivate by going through to_state
                            context = CanvasContext::new(layer_context, viewport_origin, viewport_size, canvas_size);
                        } else {
                            // Stop drawing
                            return;
                        }

                        // Pass the context on to the context
                        context.draw(&Draw::ClearCanvas);
                    }
                }

                Layer(new_layer_id) => {
                    // Extract the state from the current context (and also restore the state of the current layer)
                    let mut state = context.to_state();

                    // Update the layer ID
                    state.set_layer_id(new_layer_id);

                    // Create a new context for the layer
                    let layer_context = context_for_layer(new_layer_id);
                    if let Some(layer_context) = layer_context {
                        // Create the context for the new layer and send the state there
                        context = unsafe { CanvasContext::new(layer_context, viewport_origin, viewport_size, canvas_size) };
                        context.set_state(state);
                    } else {
                        // Stop drawing if we can't get a context for the layer
                        return;
                    }

                    // Pass the request on to the context
                    context.draw(&Draw::Layer(new_layer_id));
                },

                Store => {
                    // Store the buffer in the state
                    let layer_id = context.get_state().layer_id();

                    if let Some(existing_layer) = context.get_state().get_stored_layer() {
                        (self.update_layer)(layer_id, existing_layer);
                    } else {
                        context.get_state().clear_stored_layer();
                        context.get_state().store_layer((self.copy_layer)(layer_id));
                    }

                    // Pass on to the context (in case it needs to perform any updates relating to this action)
                    context.draw(&Draw::Store)
                }

                Restore => {
                    // Fetch the stored layer from the state
                    let layer_id        = context.get_state().layer_id();
                    let stored_buffer   = context.get_state().get_stored_layer();

                    // Restore it, if there is one
                    stored_buffer.map(|stored_buffer| (self.restore_layer)(layer_id, stored_buffer));

                    // Pass on to the context
                    context.draw(&Draw::Restore)
                }

                FreeStoredBuffer => {
                    // Clear the buffer from the state (should release it back to the layer pool)
                    context.get_state().clear_stored_layer();

                    // Pass on to the context
                    context.draw(&Draw::Restore)
                }


                // Other actions are just sent straight to the current context
                other_action => context.draw(&other_action)
            }
        }

        // Finished with the context, store the final state
        self.state = Some(context.to_state());
    }

    ///
    /// Redraws the entire canvas
    ///
    pub fn redraw<ContextForLayer: FnMut(u32) -> (Option<CFRef<CGContextRef>>)>(&mut self, context_for_layer: ContextForLayer) {
        // Fetch the current set of drawing instructions
        let mut actions = self.canvas.get_drawing();

        if actions[0] != Draw::ClearCanvas {
            actions.insert(0, Draw::ClearCanvas);
        }

        // Reset the state
        self.state = None;

        // Draw the entire canvas
        self.perform_drawing_on_context(actions, context_for_layer);
    }

    ///
    /// Draws some actions to this view canvas
    ///
    pub fn draw<ContextForLayer: FnMut(u32) -> (Option<CFRef<CGContextRef>>)>(&mut self, actions: Vec<Draw>, context_for_layer: ContextForLayer) {
        // Write the actions to the canvas so we can redraw them later on
        self.canvas.write(actions.clone());

        // Update the existing canvas context
        self.perform_drawing_on_context(actions, context_for_layer);
    }
}

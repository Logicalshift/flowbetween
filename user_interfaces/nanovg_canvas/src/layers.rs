use super::draw::*;
use super::viewport::*;
use super::framebuffer::*;

use flo_canvas::*;

use gl;
use nanovg;

use std::mem;
use std::collections::HashMap;

struct Layer {
    /// The frame buffer where this layer is stored
    frame_buffer: FrameBuffer,

    /// The context for this layer
    context: nanovg::Context
}

///
/// Performs drawing to layers stored in framebuffers
///
pub struct NanoVgLayers {
    /// The viewport, which specifes what part of the canvas to draw on the layers
    viewport: NanoVgViewport,

    /// Scale factor to apply to the viewport
    scale_factor: f32,

    /// The draw state, shared between layers
    state: NanoVgDrawingState,

    /// The stack of stored states for these layers
    state_stack: Vec<NanoVgDrawingState>,

    /// The currently selected layer
    current_layer: u32,

    /// Draw actions that are currently pending for the current layer
    pending_for_layer: Vec<Draw>,

    /// The layers, as a map of layer IDs to the framebuffers that they are stored upon
    layers: HashMap<u32, Layer>
}

impl NanoVgLayers {
    ///
    /// Creates a new layers object
    ///
    pub fn new(viewport: NanoVgViewport, scale_factor: f32) -> NanoVgLayers {
        // Create a new layers object
        NanoVgLayers {
            viewport:           viewport,
            state:              NanoVgDrawingState::new(viewport),
            current_layer:      0,
            scale_factor:       scale_factor,
            state_stack:        vec![],
            pending_for_layer:  vec![],
            layers:             HashMap::new()
        }
    }

    ///
    /// Renders the current set of layers
    ///
    pub fn render(&mut self, x: i32, y: i32) {
        // Finish any drawing that was pending
        self.flush();

        // Draw the layers to the current context
        let mut layer_ids: Vec<_> = self.layers.keys().collect();
        layer_ids.sort();

        for layer_id in layer_ids {
            self.layers[layer_id].frame_buffer.blit(x, y);
        }
    }

    ///
    /// Performs a drawing action on this layers object
    ///
    pub fn draw(&mut self, action: Draw) {
        match action {
            Draw::Layer(layer_id) => {
                // Flush actions for the current layer
                self.flush();

                // Change the layer
                self.current_layer = layer_id;
            },

            other_action => {
                // Queue up the action for later processing
                self.pending_for_layer.push(other_action)
            }
        }
    }

    ///
    /// Clears all layers and updates the viewport and scaling
    ///
    pub fn set_viewport(&mut self, new_viewport: NanoVgViewport, scale_factor: f32) {
        self.state_stack    = vec![];
        self.layers         = HashMap::new();
        self.current_layer  = 0;
        self.state          = NanoVgDrawingState::new(new_viewport.clone());
        self.viewport       = new_viewport;
        self.scale_factor   = scale_factor;
    }

    ///
    /// Clears all layers associated with this object
    ///
    pub fn clear(&mut self) {
        self.state_stack    = vec![];
        self.layers         = HashMap::new();
        self.current_layer  = 0;
        self.state          = NanoVgDrawingState::new(self.viewport.clone());
    }

    ///
    /// Flush the pending actions (eg, before changing layers)
    ///
    /// We queue up actions to avoid the need to ask nanovg
    ///
    fn flush(&mut self) {
        // Short-circuit in the event that there are no pending actions
        if self.pending_for_layer.len() == 0 {
            return;
        }

        // Bind to the GL framebuffer
        let previous_framebuffer = FrameBuffer::get_current();

        // Fetch the current layer
        let layer_id        = self.current_layer;
        let viewport        = &self.viewport;
        let scale_factor    = self.scale_factor;
        let mut actions     = vec![];
        let state           = &mut self.state;
        let layer           = self.layers.entry(layer_id).or_insert_with(|| Layer::new(viewport));

        layer.frame_buffer.bind();

        // Take the pending actions for the current layer
        mem::swap(&mut actions, &mut self.pending_for_layer);

        // Set the GL viewport for drawing
        unsafe { gl::Viewport(0, 0, viewport.viewport_width, viewport.viewport_height) };

        // Replay them to a frame
        let frame_width     = (viewport.viewport_width as f32)/scale_factor;
        let frame_height    = (viewport.viewport_height as f32)/scale_factor;
        let frame_width     = frame_width.floor();
        let frame_height    = frame_height.floor();

        layer.context.frame((frame_width, frame_height), scale_factor, move |frame| {
            for action in actions {
                Self::flush_to_layer(state, action, &frame);
            }

            state.commit(&frame);
        });

        // Reset to the framebuffer that was in use before we performed these actions
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, previous_framebuffer); }
    }

    ///
    /// Performs a draw action on the current layer
    ///
    #[inline]
    fn flush_to_layer<'a>(state: &mut NanoVgDrawingState, action: Draw, frame: &'a nanovg::Frame<'a>) {
        match action {
            // Most actions are directly processed by the layer
            other_action => state.draw(other_action, frame)
        }
    }
}

impl Layer {
    ///
    /// Creates a new layer for the specified viewport
    ///
    pub fn new(viewport: &NanoVgViewport) -> Layer {
        let framebuffer = FrameBuffer::new(viewport.viewport_width, viewport.viewport_height);
        let context     = nanovg::ContextBuilder::new()
            .build()
            .expect("Failed to build NanoVG context");

        Layer {
            frame_buffer:   framebuffer,
            context:        context
        }
    }
}

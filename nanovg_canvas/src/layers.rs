use super::draw::*;
use super::viewport::*;
use super::framebuffer::*;

use flo_canvas::*;

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
    /// Flush the pending actions (eg, before changing layers)
    /// 
    /// We queue up actions to avoid the need to ask nanovg 
    /// 
    fn flush(&mut self) {
        // Short-circuit in the event that there are no pending actions
        if self.pending_for_layer.len() == 0 {
            return;
        }

        // Fetch the current layer
        let layer_id        = self.current_layer;
        let viewport        = &self.viewport;
        let scale_factor    = self.scale_factor;
        let mut actions     = vec![];
        let state           = &mut self.state;
        let layer           = self.layers.entry(layer_id).or_insert_with(|| Layer::new(viewport));

        // Take the pending actions for the current layer
        mem::swap(&mut actions, &mut self.pending_for_layer);

        // Replay them to a frame
        layer.context.frame((viewport.viewport_width, viewport.viewport_height), scale_factor, move |frame| {
            for action in actions {
                Self::flush_to_layer(state, action, &frame);
            }
        });
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
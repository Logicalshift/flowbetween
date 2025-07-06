//!
//! The physics layer is a part of the UI that uses a physics engine to allow things to be interacted with through
//! physical properties.
//!
//! This is an experimental idea: the physics engine gives the controls a 'feeling', and also stops things like
//! controls overlapping.
//!

use super::focus::*;
use super::namespaces::*;
use super::physics_object::*;
use super::physics_tool::*;
use super::subprograms::*;

use flo_binding::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;

use futures::prelude::*;

use ::serde::*;
use serde::de::{Error as DeError};
use serde::ser::{Error as SeError};

use std::sync::*;

///
/// Instructions for the subprogram that manages the physics layer
///
pub enum PhysicsLayer {
    /// Adds a new physics tool to this layer, managed by the specified program
    AddTool(PhysicsTool, SubProgramId),

    /// Moves a tool to the LHS 'tool' dock
    DockTool(PhysicsToolId),

    /// Moves a tool to the RHS 'properties' dock
    DockProperties(PhysicsToolId),

    /// Moves a tool to a floating position
    Float(PhysicsToolId, (f64, f64)),

    /// Removes the tool with the specified ID from the physics layer
    RemoveTool(PhysicsToolId),

    /// Event to process
    Event(FocusEvent),

    /// Redraw the positions of the tools
    UpdatePositions,

    /// Redraw the sprite attached to a tool
    RedrawIcon(PhysicsToolId),
}

///
/// Events from a physics layer
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PhysicsEvent {
    /// Indicates that the specified tool has been selected
    Select(PhysicsToolId),

    /// Indicatesd that the specified tool has been deselected
    Deselect(PhysicsToolId),
}

fn test_object() -> PhysicsObject {
    let mut drawing = vec![];

    drawing.fill_color(Color::Rgba(0.0, 0.0, 1.0, 1.0));
    drawing.rect(-12.0, -12.0, 12.0, 12.0);
    drawing.fill();

    let tool = PhysicsTool::new(PhysicsToolId::new())
        .with_icon(drawing);

    PhysicsObject::new(tool, StreamTarget::None)
}

///
/// Runs the physics layer subprogram
///
pub async fn physics_layer(input: InputStream<PhysicsLayer>, context: SceneContext) {
    let our_program_id          = context.current_program_id().unwrap();
    let mut drawing_requests    = context.send::<DrawingRequest>(()).unwrap();
    let mut focus_requests      = context.send::<Focus>(()).unwrap();

    // Drawing settings
    let mut state = PhysicsLayerState {
        objects:        vec![],
        bounds:         (1024.0, 768.0),
        sprites:        vec![],
        next_sprite_id: 0,
    };

    // Objects on the layer
    let mut objects: Vec<PhysicsObject> = vec![];

    // TEST: create a test object, force an initial update
    let mut test_object = test_object();
    test_object.set_position(ToolPosition::Float(100.0, 100.0));
    state.objects.push(test_object);

    // We're a focus program with only controls, underneath pretty much anything else (so we claim z-index 0)
    focus_requests.send(Focus::ClaimRegion { program: our_program_id, region: vec![], z_index: 0 });

    // Run the main loop
    let mut input = input.ready_chunks(100);
    while let Some(request_chunk) = input.next().await {
        // What to draw for this pass through the loop
        let mut drawing                 = vec![];
        let mut positions_invalidated   = false;

        // Process the events
        for request in request_chunk.into_iter() {
            use PhysicsLayer::*;
            match request {
                AddTool(new_tool, program_id)   => { state.add_tool(new_tool, program_id); positions_invalidated = true; },
                DockTool(tool_id)               => { state.dock_tool(tool_id); positions_invalidated = true; }
                DockProperties(tool_id)         => { state.dock_properties(tool_id); positions_invalidated = true; }
                Float(tool_id, position)        => { state.float(tool_id, position); positions_invalidated = true; }
                RemoveTool(tool_id)             => { state.remove_tool(tool_id); positions_invalidated = true; }
                UpdatePositions                 => { positions_invalidated = true; }
                RedrawIcon(tool_id)             => { state.invalidate_sprite(tool_id); }

                Event(_draw_event) => {

                }
            }
        }

        // Before processing the next event, redraw the sprites for the tools
        for object in state.objects.iter_mut() {
            if object.sprite_needs_redraw() {
                // Assign a new sprite ID
                let sprite_id = if let Some(sprite) = state.sprites.pop() { sprite } else { state.next_sprite_id += 1; SpriteId(state.next_sprite_id) };

                // Add to the rendering instructions for this pass
                drawing.extend(object.draw_sprite(sprite_id, &context));

                // Positions will need to be updated at this point
                positions_invalidated = true;
            }
        }

        // Draw the tools in their expected positions
        if positions_invalidated {
            let bounds = state.bounds;

            drawing.push_state();
            drawing.namespace(*PHYSICS_LAYER);
            drawing.layer(LayerId(0));
            drawing.clear_layer();

            drawing.extend(state.objects.iter_mut().flat_map(|object| object.draw(bounds, &context)));

            drawing.pop_state();
        }

        // Send any waiting drawing instructions
        if !drawing.is_empty() {
            drawing_requests.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
        }
    }
}

///
/// State of the physics layer
///
struct PhysicsLayerState {
    /// Objects in the physics layer
    objects: Vec<PhysicsObject>,

    /// Bounds of the drawing area
    bounds: (f64, f64),

    /// The pool of sprite IDs that have been used by the tools but are now available for other uses
    sprites: Vec<SpriteId>,

    /// The sprite ID that will be assigned if no sprites are available in the pool
    next_sprite_id: u64,
}

impl PhysicsLayerState {
    ///
    /// Adds or replaces a tool within this object
    ///
    pub fn add_tool(&mut self, new_tool: PhysicsTool, target_program: SubProgramId) {
        let existing_idx = self.objects.iter().enumerate()
            .filter(|(_, object)| object.tool().id() == new_tool.id())
            .map(|(idx, _)| idx)
            .next();

        if let Some(existing_idx) = existing_idx {
            // Update the tool in the existing object
            self.objects[existing_idx].set_tool(new_tool, target_program.into());
        } else {
            // Create a new object
            let object = PhysicsObject::new(new_tool, target_program.into());
            self.objects.push(object);
        }
    }

    ///
    /// Add a tool to the end of the tool dock
    ///
    pub fn dock_tool(&mut self, tool_id: PhysicsToolId) {
        // TODO
    }

    ///
    /// Add a tool to the end of the properties dock
    ///
    pub fn dock_properties(&mut self, tool_id: PhysicsToolId) {
        // TODO
    }

    ///
    /// Sets the floating position of a tool
    ///
    pub fn float(&mut self, tool_id: PhysicsToolId, new_position: (f64, f64)) {
        let existing_idx = self.objects.iter().enumerate()
            .filter(|(_, object)| object.tool().id() == tool_id)
            .map(|(idx, _)| idx)
            .next();

        if let Some(existing_idx) = existing_idx {
            self.objects[existing_idx].set_position(ToolPosition::Float(new_position.0, new_position.1));
        }
    }

    ///
    /// Removes a tool entirely from the state
    ///
    pub fn remove_tool(&mut self, tool_id: PhysicsToolId) {
        let existing_idx = self.objects.iter().enumerate()
            .filter(|(_, object)| object.tool().id() == tool_id)
            .map(|(idx, _)| idx)
            .next();

        if let Some(existing_idx) = existing_idx {
            // Invalidate the sprite and return it to the pool
            if let Some(sprite) = self.objects[existing_idx].invalidate_sprite() {
                self.sprites.push(sprite);
            }

            // Remove from the list of objects
            self.objects.remove(existing_idx);
        }
    }

    ///
    /// Invalidates the sprite for a tool, prompting it to be redrawn
    ///
    pub fn invalidate_sprite(&mut self, tool_id: PhysicsToolId) {
        let existing_idx = self.objects.iter().enumerate()
            .filter(|(_, object)| object.tool().id() == tool_id)
            .map(|(idx, _)| idx)
            .next();

        if let Some(existing_idx) = existing_idx {
            // Invalidate the sprite and return it to the pool
            if let Some(sprite) = self.objects[existing_idx].invalidate_sprite() {
                self.sprites.push(sprite);
            }
        }
    }
}

impl Serialize for PhysicsLayer {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer 
    {
        Err(S::Error::custom("PhysicsLayer cannot be serialized"))
    }
}

impl<'a> Deserialize<'a> for PhysicsLayer {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a> 
    {
        Err(D::Error::custom("PhysicsLayer cannot be serialized"))
    }
}

impl SceneMessage for PhysicsLayer {
    fn serializable() -> bool { false }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.add_subprogram(subprogram_physics_layer(), physics_layer, 20);

        init_context.connect_programs((), subprogram_physics_layer(), StreamId::with_message_type::<PhysicsLayer>()).ok();
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|focus_events| focus_events.map(|event| PhysicsLayer::Event(event)))), (), StreamId::with_message_type::<FocusEvent>()).ok();
    }
}

impl SceneMessage for PhysicsEvent {

}

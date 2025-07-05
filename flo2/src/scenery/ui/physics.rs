//!
//! The physics layer is a part of the UI that uses a physics engine to allow things to be interacted with through
//! physical properties.
//!
//! This is an experimental idea: the physics engine gives the controls a 'feeling', and also stops things like
//! controls overlapping.
//!

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
    Event(DrawEvent),

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
    let mut drawing_requests = context.send::<DrawingRequest>(()).unwrap();

    // Drawing settings
    let mut state = PhysicsLayerState {
        objects:    vec![],
        bounds:     (1024.0, 768.0)
    };

    // Objects on the layer
    let mut objects: Vec<PhysicsObject> = vec![];

    // TEST: create a test object, force an initial update
    let mut test_object = test_object();
    test_object.set_position(ToolPosition::Float(100.0, 100.0));
    state.objects.push(test_object);

    // Sprite IDs that we're not using any more, 
    let mut sprites: Vec<SpriteId>  = vec![];
    let mut next_sprite_id          = 0;

    // Run the main loop
    let mut input = input;
    while let Some(request) = input.next().await {
        // What to draw for this pass through the loop
        let mut drawing                 = vec![];
        let mut positions_invalidated   = false;

        // Process the events
        use PhysicsLayer::*;
        match request {
            AddTool(new_tool, program_id) => { state.add_tool(new_tool, program_id); positions_invalidated = true; },

            DockTool(tool_id) => {

            }

            DockProperties(tool_id) => {

            }

            Float(tool_id, position) => {

            }

            RemoveTool(tool_id) => {

            }

            Event(_draw_event) => {

            }

            UpdatePositions => {
                positions_invalidated = true;
            }

            RedrawIcon(tool_id) => {

            }
        }

        // Before processing the next event, redraw the sprites for the tools
        for object in state.objects.iter_mut() {
            if object.sprite_needs_redraw() {
                // Assign a new sprite ID
                let sprite_id = if let Some(sprite) = sprites.pop() { sprite } else { next_sprite_id += 1; SpriteId(next_sprite_id) };

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
    }
}

impl SceneMessage for PhysicsEvent {

}

//!
//! The physics layer is a part of the UI that uses a physics engine to allow things to be interacted with through
//! physical properties.
//!
//! This is an experimental idea: the physics engine gives the controls a 'feeling', and also stops things like
//! controls overlapping.
//!

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

///
/// Runs the physics layer subprogram
///
pub async fn physics_layer(input: InputStream<PhysicsLayer>, context: SceneContext) {
    let mut drawing_requests = context.send::<DrawingRequest>(()).unwrap();

    // Objects on the layer
    let mut objects: Vec<PhysicsObject> = vec![];

    // Sprite IDs that we're not using any more, 
    let mut sprites: Vec<SpriteId>  = vec![];
    let mut next_sprite_id          = 0;

    // Run the main loop
    let mut input = input;
    while let Some(request) = input.next().await {
        // What to draw for this pass through the loop
        let mut drawing = vec![];

        // Process the events

        // Before processing the next event, redraw the sprites for the tools
        for object in objects.iter_mut() {
            if object.needs_redraw() {
                // Assign a new sprite ID
                let sprite_id = if let Some(sprite) = sprites.pop() { sprite } else { next_sprite_id += 1; SpriteId(next_sprite_id) };

                // Add to the rendering instructions for this pass
                drawing.extend(object.draw(sprite_id, &context));
            }
        }

        // Send any waiting drawing instructions
        if !drawing.is_empty() {
            drawing_requests.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
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

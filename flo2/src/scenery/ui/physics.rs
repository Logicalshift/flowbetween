//!
//! The physics layer is a part of the UI that uses a physics engine to allow things to be interacted with through
//! physical properties.
//!
//! This is an experimental idea: the physics engine gives the controls a 'feeling', and also stops things like
//! controls overlapping.
//!

use super::physics_tool::*;
use super::subprograms::*;

use flo_binding::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_scene::*;

use futures::prelude::*;

use ::serde::*;
use serde::de::{Error as DeError};
use serde::ser::{Error as SeError};

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
    // Objects on the layer
    let mut objects: Vec<PhysicsObject> = vec![];

    // Run the main loop
    let mut input = input;
    while let Some(request) = input.next().await {
        // Before processing the event, redraw the sprites for the tools

        // Process the events
    }
}

#[derive(Clone, Debug)]
enum ToolPosition {
    /// Docked to the tool bar
    DockTool,

    /// Docked to the properties bar
    DockProperties,

    /// Floating, centered at a position
    Float(f64, f64),
}

///
/// Object on the physics layer
///
struct PhysicsObject {
    /// The physics tool itself
    tool: PhysicsTool,

    /// The sprite that draws this tool (or None if there's no sprite ID)
    sprite: Option<SpriteId>,

    /// Location of the tool 
    position: Binding<ToolPosition>,

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

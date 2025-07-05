use super::binding_tracker::*;
use super::namespaces::*;
use super::physics::*;
use super::physics_tool::*;

use flo_binding::*;
use flo_binding::binding_context::*;
use flo_draw::canvas::*;
use flo_scene::*;

///
/// Location of a tool on the canvas
///
#[derive(Clone, Debug)]
pub enum ToolPosition {
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
pub struct PhysicsObject {
    /// The physics tool itself
    tool: PhysicsTool,

    /// The sprite that draws this tool (or None if there's no sprite ID)
    sprite: Option<SpriteId>,

    /// Tracker that notifies when this object's sprite needs to be redrawn
    sprite_tracker: Option<Box<dyn Releasable>>,

    /// Location of the tool 
    position: Binding<ToolPosition>,
}

impl PhysicsObject {
    ///
    /// Returns true if this object needs to be redrawn
    ///
    pub fn needs_redraw(&self) -> bool {
        self.sprite.is_none()
    }

    ///
    /// Marks this physics object as invalidated, returning the freed-up sprite ID
    ///
    pub fn invalidate(&mut self) -> Option<SpriteId> {
        // Stop tracking changes
        if let Some(mut sprite_tracker) = self.sprite_tracker.take() {
            sprite_tracker.done();
        }

        // Remove the sprite
        self.sprite.take()
    }

    ///
    /// Returns the instructions for drawing the sprite for this tool
    ///
    pub fn draw(&mut self, sprite: SpriteId, context: &SceneContext) -> Vec<Draw> {
        // Track any changes to the sprite
        let (drawing, deps) = BindingContext::bind(|| {
            let mut drawing = vec![];

            // Switch to the sprite that this tool is rendered to
            drawing.push_state();

            drawing.namespace(*PHYSICS_LAYER);
            drawing.sprite(sprite);

            // Render the tool, then switch back again
            drawing.clear_sprite();
            drawing.extend(self.tool.icon());

            drawing.pop_state();

            drawing
        });

        // Notify when the sprite changes
        self.sprite_tracker = Some(deps.when_changed(NotifySubprogram::send(PhysicsLayer::RedrawIcon(self.tool.id()), &context, ())));
        self.sprite         = Some(sprite);

        drawing
    }
}
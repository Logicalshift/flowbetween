use super::binding_tracker::*;
use super::colors::*;
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
#[derive(Clone, Debug, PartialEq)]
pub enum ToolPosition {
    // Not displayed
    Hidden,

    /// Docked to the tool bar (at the specified position)
    DockTool(usize),

    /// Docked to the properties bar (at the specified position)
    DockProperties(usize),

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
    sprite: Binding<Option<SpriteId>>,

    /// Tracker that notifies when this object's sprite needs to be redrawn
    sprite_tracker: Option<Box<dyn Releasable>>,

    /// Tracker that notifies when the position of this object has changed and the sprite/backing needs to be redrawn
    position_tracker: Option<Box<dyn Releasable>>,

    /// Location of the tool 
    position: Binding<ToolPosition>,
}

impl PhysicsObject {
    ///
    /// Creates a new hidden physics tool
    ///
    pub fn new(tool: PhysicsTool) -> Self {
        Self {
            tool:               tool,
            sprite:             bind(None),
            sprite_tracker:     None,
            position_tracker:   None,
            position:           bind(ToolPosition::Hidden),
        }
    }

    ///
    /// Returns true if this object needs to be redrawn
    ///
    pub fn sprite_needs_redraw(&self) -> bool {
        self.sprite.get().is_none()
    }

    ///
    /// Marks this physics object as invalidated, returning the freed-up sprite ID
    ///
    pub fn invalidate_sprite(&mut self) -> Option<SpriteId> {
        // Stop tracking changes
        if let Some(mut sprite_tracker) = self.sprite_tracker.take() {
            sprite_tracker.done();
        }

        // Remove the sprite
        let sprite = self.sprite.get();
        self.sprite.set(None);
        sprite
    }

    ///
    /// Returns the instructions for drawing the sprite for this tool
    ///
    pub fn draw_sprite(&mut self, sprite: SpriteId, context: &SceneContext) -> Vec<Draw> {
        // Avoid sending any sprite updates that predate this update
        if let Some(mut sprite_tracker) = self.sprite_tracker.take() {
            sprite_tracker.done();
        }

        // Assume we'll update the position too
        if let Some(mut position_tracker) = self.position_tracker.take() {
            position_tracker.done();
        }

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
        self.sprite_tracker = Some(deps.when_changed(NotifySubprogram::send(PhysicsLayer::RedrawIcon(self.tool.id()), context, ())));
        self.sprite.set(Some(sprite));

        drawing
    }

    ///
    /// Sets the position of this object
    ///
    pub fn set_position(&mut self, new_position: ToolPosition) {
        self.position.set(new_position);
    }

    ///
    /// Returns the coordinates where the center of this object should be rendered
    ///
    pub fn position(&self, bounds: (f64, f64)) -> Option<(f64, f64)> {
        match self.position.get() {
            ToolPosition::Hidden                => None,
            ToolPosition::DockTool(idx)         => Some((20.0, 20.0 + (idx as f64 * 40.0))),
            ToolPosition::DockProperties(idx)   => Some((bounds.0 - 20.0, 20.0 + (idx as f64 * 40.0))),
            ToolPosition::Float(x, y)           => Some((x, y)),
        }
    }

    ///
    /// Returns the instructions to draw this physics object
    ///
    pub fn draw(&mut self, bounds: (f64, f64), context: &SceneContext) -> Vec<Draw> {
        if let Some(mut position_tracker) = self.position_tracker.take() {
            position_tracker.done();
        }

        // Changes to the position get tracked
        let (drawing, deps) = BindingContext::bind(|| {
            let mut drawing = vec![];

            // Determine the position of this control
            let pos         = self.position(bounds);
            let has_shadow  = match self.position.get() {
                ToolPosition::Hidden            |
                ToolPosition::DockTool(_)       |
                ToolPosition::DockProperties(_) => false,
                ToolPosition::Float(_, _)       => true,
            };

            let pos     = if let Some(pos) = pos { pos } else { return drawing; };
            let sprite  = self.sprite.get();
            let sprite  = if let Some(sprite) = sprite { sprite } else { return drawing; };
            let (x, y)  = pos;
            let (w, h)  = self.tool.size();

            // Render the backing circle
            if has_shadow {
                drawing.new_path();
                drawing.circle(x as f32 + 1.0, y as f32 + 3.0, (w.max(h)/2.0) as f32);
                drawing.fill_color(color_tool_shadow());
                drawing.fill();
            }

            drawing.new_path();
            drawing.circle(x as f32, y as f32, (w.max(h)/2.0 - 2.0) as f32);
            drawing.fill_color(color_tool_background());
            drawing.stroke_color(color_tool_outline());
            drawing.line_width(2.0);
            drawing.fill();
            drawing.stroke();

            drawing.circle(x as f32, y as f32, (w.max(h)/2.0) as f32);
            drawing.stroke_color(color_tool_border());
            drawing.line_width(1.0);
            drawing.stroke();

            // Render the sprite to draw the actual physics object
            drawing.sprite_transform(SpriteTransform::Identity);
            drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));
            drawing.draw_sprite(sprite);

            drawing
        });

        // Notify when the position changes
        self.position_tracker = Some(deps.when_changed(NotifySubprogram::send(PhysicsLayer::UpdatePositions, context, ())));

        drawing
    }
}
use super::blobland::*;
use super::physics::*;
use super::physics_tool::*;
use crate::scenery::ui::binding_tracker::*;
use crate::scenery::ui::focus::*;
use crate::scenery::ui::namespaces::*;
use crate::scenery::ui::ui_path::*;

use futures::prelude::*;

use flo_binding::*;
use flo_binding::binding_context::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_scene::*;

use ::serde::*;

///
/// Object on the physics layer
///
pub struct PhysicsObject {
    /// The physics tool itself
    tool: PhysicsTool,

    /// The subprogram ID of the program that manages the events for this control
    subprogram_id: SubProgramId,

    /// Where events for this tool should be sent
    event_target: StreamTarget,

    /// The sprite that draws this tool (or None if there's no sprite ID)
    sprite: Binding<Option<SpriteId>>,

    /// Tracker that notifies when this object's sprite needs to be redrawn
    sprite_tracker: Option<Box<dyn Releasable>>,

    /// Tracker that notifies when the position of this object has changed and the sprite/backing needs to be redrawn
    position_tracker: Option<Box<dyn Releasable>>,

    /// Location of the tool 
    position: Binding<ToolPosition>,

    /// The offset from the position that this was last dragged to
    position_offset: Binding<UiPoint>,

    /// True if this tool is being dragged
    being_dragged: bool,

    /// The drag anchor specified by the last 'start drag' operation
    drag_anchor: UiPoint,

    /// The drag position of this object (as opposed to the 'real' position that it assumes when the drag has finished)
    drag_position: Binding<Option<UiPoint>>,

    /// The ID of this tool in the blobland
    blob_id: BlobId,
}

///
/// Represents an action performed on a physics tool
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PhysicsObjectAction {
    /// Pick this tool as the active tool
    Activate(PhysicsToolId),

    /// Expand the settings for this tool
    Expand(PhysicsToolId),

    /// Physics tool is being dragged
    StartDrag(PhysicsToolId, f64, f64),

    /// Move the tool by the specified offset from the StartDrag
    Drag(PhysicsToolId, f64, f64),

    /// Finishes a drag
    EndDrag(PhysicsToolId, f64, f64),
}

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

impl PhysicsObject {
    ///
    /// Creates a new hidden physics tool
    ///
    pub fn new(tool: PhysicsTool, event_target: StreamTarget) -> Self {
        Self {
            tool:               tool,
            subprogram_id:      SubProgramId::new(),
            event_target:       event_target,
            sprite:             bind(None),
            sprite_tracker:     None,
            position_tracker:   None,
            position:           bind(ToolPosition::Hidden),
            position_offset:    bind(UiPoint(0.0, 0.0)),
            being_dragged:      false,
            drag_anchor:        UiPoint(0.0, 0.0),
            drag_position:      bind(None),
            blob_id:            BlobId::new(),
        }
    }

    ///
    /// Adds a blob for this tool to a BlobLand
    ///
    pub fn add_blob(&mut self, blob_land: &mut BlobLand, bounds: (f64, f64), interaction: impl 'static + Send + Fn(BlobId) -> BlobInteraction) -> BlobId {
        let pos     = self.position(bounds).unwrap_or(UiPoint(0.0, 0.0));
        let (w, h)  = self.tool.size();
        let radius  = w.min(h)/2.0;
        let blob    = Blob::new(UiPoint(pos.0, pos.1), radius * 1.5, radius).with_interaction(interaction);

        self.blob_id = blob.id();
        blob_land.add_blob(blob);

        self.blob_id
    }

    ///
    /// Returns the blob ID assigned to this physics object
    ///
    #[inline]
    pub fn blob_id(&self) -> BlobId {
        self.blob_id
    }

    ///
    /// Retrieves the physics tool that is being managed by this object
    ///
    pub fn tool(&self) -> &PhysicsTool {
        &self.tool
    }

    ///
    /// The control ID that represents this object on the canvas
    ///
    pub fn subprogram_id(&self) -> SubProgramId {
        self.subprogram_id
    }

    ///
    /// Replaces the tool represented by this object
    ///
    pub fn set_tool(&mut self, new_tool: PhysicsTool, new_target: StreamTarget) {
        self.tool           = new_tool;
        self.event_target   = new_target;
        self.invalidate_sprite();
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
    /// Sets the position of this object
    ///
    pub fn update_blob_position(&mut self, blob_land: &mut BlobLand, bounds: (f64, f64)) {
        let new_pos = self.drag_position.get().or_else(|| self.position(bounds));
        if let Some(new_pos) = new_pos {
            blob_land.move_blob(self.blob_id, UiPoint(new_pos.0, new_pos.1));
        }
    }

    ///
    /// Returns the coordinates where the center of this object should be rendered
    ///
    pub fn position(&self, bounds: (f64, f64)) -> Option<UiPoint> {
        match self.position.get() {
            ToolPosition::Hidden                => None,
            ToolPosition::DockTool(idx)         => Some(UiPoint(20.0, 20.0 + (idx as f64 * 40.0))),
            ToolPosition::DockProperties(idx)   => Some(UiPoint(bounds.0 - 20.0, 20.0 + (idx as f64 * 40.0))),
            ToolPosition::Float(x, y)           => Some(UiPoint(x, y) + self.position_offset.get()),
        }
    }

    ///
    /// Starts a drag operation on this object
    ///
    pub fn start_drag(&mut self, x: f64, y: f64, bounds: (f64, f64)) {
        self.being_dragged = true;

        // Anchor at the position the tool was in originally
        self.drag_anchor = UiPoint(x, y);
        self.drag_position.set(self.position(bounds));

        // Remove the offset
        self.position_offset.set(UiPoint(0.0, 0.0));
    }

    ///
    /// Starts a drag operation on this object
    ///
    pub fn drag(&mut self, x: f64, y: f64) {
        if let Some(UiPoint(x_pos, y_pos)) = self.drag_position.get() {
            // Calculate the offset from the existing drag anchor
            let offset_x = x - self.drag_anchor.0;
            let offset_y = y - self.drag_anchor.1;

            // Move the drag position by the offset
            self.drag_position.set(Some(UiPoint(x_pos + offset_x, y_pos + offset_y)));

            // Update the anchor
            self.drag_anchor.0 += offset_x;
            self.drag_anchor.1 += offset_y;
        }
    }

    ///
    /// Finishes a drag operation on this object
    ///
    pub fn end_drag(&mut self, x: f64, y: f64) {
        if let Some(UiPoint(new_x, new_y)) = self.drag_position.get() {
            // Calculate the offset from the existing drag anchor
            let offset_x = x - self.drag_anchor.0;
            let offset_y = y - self.drag_anchor.1;

            let new_x = new_x + offset_x;
            let new_y = new_y + offset_y;

            // Set this as the final position of the tool
            self.set_position(ToolPosition::Float(new_x, new_y));
            self.drag_position.set(None);
            self.being_dragged = false;
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
            let pos             = self.position(bounds);
            let pos             = if let Some(pos) = pos { pos } else { return drawing; };
            let pos             = if let Some(drag_position) = self.drag_position.get() { drag_position } else { pos };
            let sprite          = self.sprite.get();
            let sprite          = if let Some(sprite) = sprite { sprite } else { return drawing; };
            let UiPoint(x, y)   = pos;

            // Render the sprite to draw the actual physics object
            drawing.sprite_transform(SpriteTransform::Identity);
            drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));
            drawing.draw_sprite(sprite);

            drawing
        });

        // Notify when the position changes
        self.position_tracker = Some(deps.when_changed(NotifySubprogram::send(PhysicsLayer::UpdatePosition(self.tool.id()), context, ())));

        drawing
    }
}

///
/// Runs a drag action for a physics object tool
///
async fn track_drag((x, y): (f64, f64), input: &mut InputStream<FocusEvent>, tool_id: PhysicsToolId, layer_actions: &mut OutputSink<PhysicsLayer>) {
    let mut last_x = x;
    let mut last_y = y;

    // Send the start drag action
    layer_actions.send(PhysicsLayer::ObjectAction(PhysicsObjectAction::StartDrag(tool_id, x, y))).await.ok();

    // Track the pointer as it moves during the drag, until the button is released
    while let Some(evt) = input.next().await {
        match evt {
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Move, _, pointer_state)) => {
                if let Some((x, y)) = pointer_state.location_in_canvas {
                    // Continue the drag
                    layer_actions.send(PhysicsLayer::ObjectAction(PhysicsObjectAction::Drag(tool_id, x, y))).await.ok();

                    last_x = x;
                    last_y = y;
                }
            }

            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::Drag, _, pointer_state)) => {
                if let Some((x, y)) = pointer_state.location_in_canvas {
                    // Continue the drag
                    layer_actions.send(PhysicsLayer::ObjectAction(PhysicsObjectAction::Drag(tool_id, x, y))).await.ok();

                    last_x = x;
                    last_y = y;
                }
            }

            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::ButtonUp, _, pointer_state)) => {
                // Finish the drag
                if let Some((x, y)) = pointer_state.location_in_canvas {
                    layer_actions.send(PhysicsLayer::ObjectAction(PhysicsObjectAction::EndDrag(tool_id, x, y))).await.ok();
                } else {
                    layer_actions.send(PhysicsLayer::ObjectAction(PhysicsObjectAction::EndDrag(tool_id, last_x, last_y))).await.ok();
                }

                // Exit the drag loop
                break;
            }

            _ => { }
        }
    }
}

///
/// Subprogram that manages the basic mouse and keyboard events for a PhysicsObject
///
/// This will generate PhysicsLayerEvents, and expects to receive DrawEvents from the focus subprogram (it relies on the focus subprogram
/// to route events to it)
///
pub async fn physics_object_program(input: InputStream<FocusEvent>, context: SceneContext, tool_id: PhysicsToolId) {
    let mut layer_actions   = context.send(()).unwrap();
    let mut input           = input;

    while let Some(evt) = input.next().await {
        match evt {
            FocusEvent::Event(_, DrawEvent::Pointer(PointerAction::ButtonDown, _, pointer_state)) => {
                if let Some((x, y)) = pointer_state.location_in_canvas {
                    // Drag the tool
                    track_drag((x, y), &mut input, tool_id, &mut layer_actions).await;
                }
            }

            _ => { }
        }
    }
}

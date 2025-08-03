use super::blobland::*;
use super::physics_layer::*;
use super::physics_simulation::*;
use super::physics_tool::*;
use crate::scenery::ui::binding_program::*;
use crate::scenery::ui::binding_tracker::*;
use crate::scenery::ui::focus::*;
use crate::scenery::ui::namespaces::*;
use crate::scenery::ui::ui_path::*;

use futures::prelude::*;
use futures::future::{BoxFuture};

use flo_curves::arc::*;
use flo_binding::*;
use flo_binding::binding_context::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_scene::*;

use ::serde::*;

use std::sync::*;

///
/// Object on the physics layer
///
pub struct PhysicsObject {
    /// The physics tool itself
    tool: PhysicsTool,

    /// The rendering properties for this object
    properties: Arc<PhysicsObjectProperties>,

    /// The subprogram ID of the program that manages the events for this control
    subprogram_id: SubProgramId,

    /// The ID of this object within the physics simulation
    physics_id: SimObjectId,

    /// Where events for this tool should be sent
    event_target: StreamTarget,

    /// Tracker that notifies when this object's sprite needs to be redrawn
    sprite_tracker: Option<Box<dyn Releasable>>,

    /// Location of the tool (ideal, before the physics engine)
    position: Binding<ToolPosition>,

    /// True if this tool is being dragged
    being_dragged: bool,

    /// The drag anchor specified by the last 'start drag' operation
    drag_anchor: UiPoint,
}

///
/// Describes the shared properties for a physics object
///
/// This is a set of bindings, as the rendering is updated by a render binding program (and also so this can be wrapped in an 'Arc' so we can send the properties
/// all in one go, without a large amount of cloning)
///
#[derive(Clone, PartialEq)]
pub struct PhysicsObjectProperties {
    /// Whether or not the object is hidden
    hidden: Binding<bool>,

    /// The position of the object (managed by the physics program)
    position: Binding<UiPoint>,

    /// The position that the user has dragged this object to
    drag_position: Binding<Option<UiPoint>>,

    /// The sprite ID of the icon for this physics object
    sprite: Binding<Option<SpriteId>>,

    /// The ID of the blob that should be rendered from the blobland
    blob_id: Binding<BlobId>,
}

impl PhysicsObjectProperties {
    ///
    /// Creates a new physics object properties structure
    ///
    pub fn new() -> Self {
        Self {
            hidden:         bind(false),
            position:       bind(UiPoint(0.0, 0.0)),
            drag_position:  bind(None),
            sprite:         bind(None),
            blob_id:        bind(BlobId::new()),
        }
    }

    ///
    /// The position of this physics object
    ///
    #[inline]
    pub fn position(&self) -> UiPoint {
        self.position.get()
    }

    ///
    /// If the object is being dragged, the position that it's currently dragged to
    ///
    #[inline]
    pub fn drag_position(&self) -> Option<UiPoint> {
        self.drag_position.get()
    }

    ///
    /// Draws the object with these properties at the specified position
    ///
    pub fn draw(&self, drawing: &mut impl GraphicsContext, blob_land: &mut BlobLand) {
        // Nothing to draw if this object is hidden
        if self.hidden.get() {
            return;
        }

        // Fetch the property values
        let position        = self.position.get();
        let drag_position   = self.drag_position.get();
        let blob_id         = self.blob_id.get();
        let sprite          = self.sprite.get();

        // The drag position overrides the position
        let position        = if let Some(drag_position) = drag_position { drag_position } else { position };

        // Update the blob land with the actual position of this object (blob land needs to be rendered as a whole, which we assume we do later on)
        blob_land.move_blob(blob_id, position);

        if let Some(sprite) = sprite {
            // Render the sprite to draw the actual physics object
            let UiPoint(x, y)   = position;

            drawing.sprite_transform(SpriteTransform::Identity);
            drawing.sprite_transform(SpriteTransform::Translate(x as f32, y as f32));
            drawing.draw_sprite(sprite);
        }
    }
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
            properties:         Arc::new(PhysicsObjectProperties::new()),
            subprogram_id:      SubProgramId::new(),
            physics_id:         SimObjectId::new(),
            event_target:       event_target,
            sprite_tracker:     None,
            position:           bind(ToolPosition::Hidden),
            being_dragged:      false,
            drag_anchor:        UiPoint(0.0, 0.0),
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

        self.properties.blob_id.set(blob.id());
        blob_land.add_blob(blob);

        self.properties.blob_id.get()
    }

    ///
    /// Returns the render properties for this object
    ///
    #[inline]
    pub fn render_properties(&self) -> &Arc<PhysicsObjectProperties> {
        &self.properties
    }

    ///
    /// Returns the blob ID assigned to this physics object
    ///
    #[inline]
    pub fn blob_id(&self) -> BlobId {
        self.properties.blob_id.get()
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
        self.properties.sprite.get().is_none()
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
        let sprite = self.properties.sprite.get();
        self.properties.sprite.set(None);
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
        self.properties.sprite.set(Some(sprite));

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
    pub fn position(&self, bounds: (f64, f64)) -> Option<UiPoint> {
        match self.position.get() {
            ToolPosition::Hidden                => None,
            ToolPosition::DockTool(idx)         => Some(UiPoint(20.0, 20.0 + (idx as f64 * 40.0))),
            ToolPosition::DockProperties(idx)   => Some(UiPoint(bounds.0 - 20.0, 20.0 + (idx as f64 * 40.0))),
            ToolPosition::Float(x, y)           => Some(UiPoint(x, y)),
        }
    }

    ///
    /// Creates this object in the physics simulation
    ///
    pub fn create_in_simulation(&self, bounds: (f64, f64), requests: &mut OutputSink<PhysicsSimulation>) -> impl Send + Future<Output=()> {
        let physics_id          = self.physics_id;
        let position            = self.position(bounds);
        let tool_size           = self.tool.size();
        let position_binding    = self.properties.position.clone();

        async move {
            // Create the body
            requests.send(PhysicsSimulation::CreateRigidBody(physics_id)).await.ok();

            // Set up the position binding (will connect to the renderer)
            requests.send(PhysicsSimulation::BindPosition(physics_id, position_binding)).await.ok();

            // Set the initial position and shape of the object
            requests.send(PhysicsSimulation::Set(physics_id, vec![
                SimBodyProperty::Position(position.unwrap_or(UiPoint(0.0, 0.0))),
                SimBodyProperty::Type(SimObjectType::Dynamic),
                SimBodyProperty::Shape(SimShape::Circle(tool_size.0))
            ])).await.ok();
        }
    }

    ///
    /// Updates the position of this object in a simulation
    ///
    pub fn update_in_simulation<'a>(&self, bounds: (f64, f64), requests: &'a mut OutputSink<PhysicsSimulation>) -> BoxFuture<'a, ()> {
        let physics_id      = self.physics_id;
        let position        = if let Some(drag_position) = self.properties.drag_position() { Some(drag_position) } else { self.position(bounds) };
        let tool_size       = self.tool.size();
        let being_dragged   = self.being_dragged;

        async move {
            let object_type = if being_dragged { SimObjectType::Kinematic } else { SimObjectType::Dynamic };

            requests.send(PhysicsSimulation::Set(physics_id, vec![
                SimBodyProperty::Type(object_type),
                SimBodyProperty::Position(position.unwrap_or(UiPoint(0.0, 0.0))),
                SimBodyProperty::Shape(SimShape::Circle(tool_size.0))
            ])).await.ok();
        }.boxed()
    }

    ///
    /// Starts a drag operation on this object
    ///
    pub fn start_drag(&mut self, x: f64, y: f64) {
        self.being_dragged = true;

        // Anchor at the position the tool was in originally
        self.drag_anchor = UiPoint(x, y);
        self.properties.drag_position.set(Some(self.properties.position()));
    }

    ///
    /// Starts a drag operation on this object
    ///
    pub fn drag(&mut self, x: f64, y: f64) {
        if let Some(UiPoint(x_pos, y_pos)) = self.properties.drag_position.get() {
            // Calculate the offset from the existing drag anchor
            let offset_x = x - self.drag_anchor.0;
            let offset_y = y - self.drag_anchor.1;

            // Move the drag position by the offset
            self.properties.drag_position.set(Some(UiPoint(x_pos + offset_x, y_pos + offset_y)));

            // Update the anchor
            self.drag_anchor.0 += offset_x;
            self.drag_anchor.1 += offset_y;
        }
    }

    ///
    /// Finishes a drag operation on this object
    ///
    pub fn end_drag(&mut self, x: f64, y: f64) {
        if let Some(UiPoint(new_x, new_y)) = self.properties.drag_position.get() {
            // Calculate the offset from the existing drag anchor
            let offset_x = x - self.drag_anchor.0;
            let offset_y = y - self.drag_anchor.1;

            let new_x = new_x + offset_x;
            let new_y = new_y + offset_y;

            // Set this as the final position of the tool
            self.set_position(ToolPosition::Float(new_x, new_y));
            self.properties.drag_position.set(None);
            self.being_dragged = false;
        }
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
pub async fn physics_object_mouse_program(input: InputStream<FocusEvent>, context: SceneContext, tool_id: PhysicsToolId) {
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

///
/// Subprogram that causes the 'focus' to follow the mouse
///
pub async fn physics_object_focus_program(input: InputStream<BindingProgram>, context: SceneContext, mouse_program: SubProgramId, position: impl Into<BindRef<UiPoint>>, size: impl Into<BindRef<(f64, f64)>>) {
    // Create a binding for the position and the size
    let position        = position.into();
    let size            = size.into();
    let position_size   = computed(move || (position.get(), size.get()));

    // Run a binding program to send focus updates whenever the position changes
    let action = BindingAction::<(UiPoint, (f64, f64)), _, _>::new(move |(position, (size_w, size_h)), context| {
            let context = context.clone();

            async move {
                // Set up the path for the tool
                let tool_size   = size_w.max(size_h);
                let tool_path   = Circle::new(position, tool_size/2.0).to_path();

                // Update the region that the tool is occupying
                context.send_message(Focus::ClaimRegion {
                    program:    mouse_program,
                    region:     vec![tool_path],
                    z_index:    1,
                }).await.unwrap();
            }
        })
        .with_parent_program(mouse_program)
        .with_stop_action(move |context| async move {
            // Remove the focus claim when this program stops (which is also when the mouse program stops)
            context.send_message(Focus::RemoveClaim(mouse_program)).await.unwrap();
        }.boxed());

    binding_program(input, context, position_size, action).await;
}

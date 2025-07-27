//!
//! The physics layer is a part of the UI that uses a physics engine to allow things to be interacted with through
//! physical properties.
//!
//! This is an experimental idea: the physics engine gives the controls a 'feeling', and also stops things like
//! controls overlapping.
//!

use super::blobland::*;
use super::physics_object::*;
use super::physics_tool::*;
use super::physics_simulation::*;
use crate::scenery::ui::colors::*;
use crate::scenery::ui::focus::*;
use crate::scenery::ui::namespaces::*;
use crate::scenery::ui::render_binding::*;
use crate::scenery::ui::subprograms::*;
use crate::scenery::ui::ui_path::*;

use flo_binding::*;
use flo_draw::*;
use flo_draw::canvas::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_curves::arc::*;

use futures::prelude::*;

use ::serde::*;
use serde::de::{Error as DeError};
use serde::ser::{Error as SeError};

use std::collections::{HashMap};
use std::sync::*;
use std::time::{Instant, Duration};

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

    /// Redraw the positions of a particular tool
    UpdatePosition(PhysicsToolId),

    /// Redraw the sprite attached to a tool
    RedrawIcon(PhysicsToolId),

    /// Action message for a physics object
    ObjectAction(PhysicsObjectAction),
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

fn test_tool() -> PhysicsTool {
    let mut drawing = vec![];

    drawing.fill_color(Color::Rgba(0.0, 0.0, 1.0, 1.0));
    drawing.rect(-12.0, -12.0, 12.0, 12.0);
    drawing.fill();

    let tool = PhysicsTool::new(PhysicsToolId::new())
        .with_icon(drawing);

    tool
}

///
/// Runs the physics layer subprogram
///
pub async fn physics_layer(input: InputStream<PhysicsLayer>, context: SceneContext) {
    let our_program_id          = context.current_program_id().unwrap();

    // Start a physics subprogram for this layer
    let physics_program_id      = SubProgramId::new();
    context.send_message(SceneControl::start_program(physics_program_id,
        move |input, context| physics_simulation_program(input, context, true),
        100
    )).await.unwrap();

    // Connect to the events
    let mut drawing_requests    = context.send::<DrawingRequest>(()).unwrap();
    let mut focus_requests      = context.send::<Focus>(()).unwrap();
    let mut physics_requests    = context.send::<PhysicsSimulation>(physics_program_id).unwrap();

    // Drawing settings
    let render_state = PhysicsLayerRenderState {
        blob_land:          BlobLand::empty(),
        blob_tick:          Instant::now(),
        blob_awake:         false,
        object_properties:  bind(Arc::new(vec![])),
    };

    let mut state = PhysicsLayerState {
        our_program_id:         our_program_id,
        objects:                Arc::new(Mutex::new(HashMap::new())),
        render_state:           Arc::new(Mutex::new(render_state)),
        bounds:                 (1024.0, 768.0),
        tool_id_for_blob_id:    Arc::new(Mutex::new(HashMap::new())),
        sprites:                vec![],
        next_sprite_id:         0,
    };

    // Create rendering program
    state.start_rendering_program(&context).await;

    // TEST: create a test object, force an initial update
    let mut test_object = test_tool();
    let test_object_id = test_object.id();
    state.add_tool(test_object, StreamTarget::None, &context).await;
    state.float(test_object_id, (100.0, 100.0));
    state.update_tool_focus(test_object_id, &mut focus_requests).await;

    let mut test_object = test_tool();
    let test_object_id = test_object.id();
    state.add_tool(test_object, StreamTarget::None, &context).await;
    state.float(test_object_id, (200.0, 100.0));
    state.update_tool_focus(test_object_id, &mut focus_requests).await;

    let mut test_object = test_tool();
    let test_object_id = test_object.id();
    state.add_tool(test_object, StreamTarget::None, &context).await;
    state.float(test_object_id, (300.0, 100.0));
    state.update_tool_focus(test_object_id, &mut focus_requests).await;

    // We're a focus program with only controls, underneath pretty much anything else (so we claim z-index 0)
    focus_requests.send(Focus::ClaimRegion { program: our_program_id, region: vec![], z_index: 0 }).await.ok();

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
                // Tool requests
                AddTool(new_tool, program_id)   => { let tool_id = new_tool.id(); state.add_tool(new_tool, program_id.into(), &context).await; positions_invalidated = true; state.update_tool_focus(tool_id, &mut focus_requests).await; },
                DockTool(tool_id)               => { state.dock_tool(tool_id); positions_invalidated = true; state.update_tool_focus(tool_id, &mut focus_requests).await; }
                DockProperties(tool_id)         => { state.dock_properties(tool_id); positions_invalidated = true; state.update_tool_focus(tool_id, &mut focus_requests).await; }
                Float(tool_id, position)        => { state.float(tool_id, position); positions_invalidated = true; state.update_tool_focus(tool_id, &mut focus_requests).await; }
                RemoveTool(tool_id)             => { state.remove_tool(tool_id); positions_invalidated = true; }
                UpdatePosition(tool_id)         => { state.update_tool_focus(tool_id, &mut focus_requests).await; positions_invalidated = true; }
                RedrawIcon(tool_id)             => { state.invalidate_sprite(tool_id); }

                // Event handling
                Event(FocusEvent::Event(_, DrawEvent::Resize(w, h)))   => { state.set_bounds(w, h); positions_invalidated = true; }

                Event(_draw_event) => { }

                ObjectAction(PhysicsObjectAction::Activate(tool_id))        => { }
                ObjectAction(PhysicsObjectAction::Expand(tool_id))          => { }
                ObjectAction(PhysicsObjectAction::StartDrag(tool_id, x, y)) => { let bounds = state.bounds; state.object_action(tool_id, |object, _| object.start_drag(x, y, bounds)); }
                ObjectAction(PhysicsObjectAction::Drag(tool_id, x, y))      => { state.object_action(tool_id, |object, _| { object.drag(x, y); }); }
                ObjectAction(PhysicsObjectAction::EndDrag(tool_id, x, y))   => { state.object_action(tool_id, |object, _| { object.end_drag(x, y); }); }
            }
        }

        // Before processing the next event, redraw the sprites for the tools
        for object in state.objects.lock().unwrap().values_mut() {
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
        /*
        if positions_invalidated {
            // Simulation needs to run/restart if the positions are invalidated
            state.run_simulation(&mut drawing_requests).await;

            drawing.push_state();
            drawing.namespace(*PHYSICS_LAYER);
            drawing.layer(LayerId(1));
            drawing.clear_layer();

            let objects     = &mut state.objects;
            let blob_land   = &mut state.blob_land;

            objects.lock().unwrap()
                .values_mut()
                .for_each(|object| {
                    object.render_properties().draw(&mut drawing, blob_land);
                });

            drawing.pop_state();
        }

        // Send any waiting drawing instructions
        if !drawing.is_empty() {
            drawing_requests.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
        }
        */
    }

    // Stop the physics program when we're finished
    context.send_message(SceneControl::Close(physics_program_id)).await.ok();
}

///
/// The render state of the physics layer is used to communicate with the rendering subprogram
///
struct PhysicsLayerRenderState {
    /// The blobland simulation that causes the tools to animate when they're joined together or pulled apart
    blob_land: BlobLand,

    /// The instant when the last simulation tick was run
    blob_tick: Instant,

    /// True if the simulation is awake
    blob_awake: bool,

    /// The properties of the objects that are on the physics layer
    object_properties: Binding<Arc<Vec<Arc<PhysicsObjectProperties>>>>,
}

///
/// State of the physics layer
///
struct PhysicsLayerState {
    // Program ID of the phyics layer program
    our_program_id: SubProgramId,

    /// Objects in the physics layer
    objects: Arc<Mutex<HashMap<PhysicsToolId, PhysicsObject>>>,

    /// Describes how the layer should be rendered
    render_state: Arc<Mutex<PhysicsLayerRenderState>>,

    /// The tool ID associated with a blob ID from the blobland
    tool_id_for_blob_id: Arc<Mutex<HashMap<BlobId, PhysicsToolId>>>,

    /// Bounds of the drawing area
    bounds: (f64, f64),

    /// The pool of sprite IDs that have been used by the tools but are now available for other uses
    sprites: Vec<SpriteId>,

    /// The sprite ID that will be assigned if no sprites are available in the pool
    next_sprite_id: u64,
}

impl PhysicsLayerState {
    ///
    /// Creates the blob interaction function for a new blob
    ///
    fn blob_interaction_fn<'a>(our_tool_id: PhysicsToolId, objects: &'a Arc<Mutex<HashMap<PhysicsToolId, PhysicsObject>>>, tool_id_for_blob_id: &'a Arc<Mutex<HashMap<BlobId, PhysicsToolId>>>) -> impl 'static + Send + Fn(BlobId) -> BlobInteraction {
        let objects             = objects.clone();
        let tool_id_for_blob_id = tool_id_for_blob_id.clone();

        // Blobs attract each other if they can be combined
        move |blob_id| {
            let other_tool_id = tool_id_for_blob_id.lock().unwrap().get(&blob_id).copied();

            if let Some(other_tool_id) = other_tool_id {
                let objects = objects.lock().unwrap();

                if let (Some(our_tool), Some(other_tool)) = (objects.get(&our_tool_id), objects.get(&other_tool_id)) {
                    // The tools attract if they are in groups that can attract
                    // TODO: docked tools aren't attractive
                    let other_tool_group = other_tool.tool().selection_group();

                    if our_tool.tool().will_bind_with(other_tool_group) {
                        BlobInteraction::Attract
                    } else {
                        BlobInteraction::Repel
                    }
                } else {
                    // No interaction if the tools are missing
                    BlobInteraction::None
                }
            } else {
                // No interaction if the tool can't be looked up
                BlobInteraction::None
            }
        }
    }

    ///
    /// Creates a rendering program to display this state
    ///
    async fn start_rendering_program(&self, context: &SceneContext) {
        let render_state    = self.render_state.clone();
        let our_program_id  = self.our_program_id;

        context.send_message(SceneControl::start_program(SubProgramId::new(), move |input, context| {
            render_binding_program(input, context, (*PHYSICS_LAYER, LayerId(0)), Some(our_program_id), computed(move || {
                let mut render_state    = render_state.lock().unwrap();
                let mut rendering       = vec![];

                // Render the decals
                let object_properties = render_state.object_properties.get();

                for obj in object_properties.iter() {
                    obj.draw(&mut rendering, &mut render_state.blob_land);
                }

                // The borders are rendered using the 'BlobLand', which deals with the animations that join related tools together
                let now             = Instant::now();
                let tick_time       = now.duration_since(render_state.blob_tick);
                let tick_time_us    = tick_time.as_micros();
                let tick_time_s     = (tick_time_us as f64) / 1_000_000.0;

                // Run a frame of simulation for the blobland
                render_state.blob_land.simulate(tick_time_s);
                render_state.blob_tick = now;

                // Draw the blobs
                let mut blob_drawing = vec![];

                blob_drawing.fill_color(color_tool_background());
                blob_drawing.stroke_color(color_tool_outline());
                blob_drawing.line_width(2.0);
                render_state.blob_land.render(&mut blob_drawing);

                // Add to the start of the drawing
                rendering.splice(0..0, blob_drawing);

                rendering
            }).into())
        }, 1)).await.ok();
    }

    ///
    /// Adds or replaces a tool within this object
    ///
    pub async fn add_tool(&mut self, new_tool: PhysicsTool, target_program: StreamTarget, context: &SceneContext) {
        if let Some(existing_object) = self.objects.lock().unwrap().get_mut(&new_tool.id()) {
            // Update the tool in the existing object
            existing_object.set_tool(new_tool, target_program.into());
        } else {
            // Create a new object
            let tool_id             = new_tool.id();
            let mut object          = PhysicsObject::new(new_tool, target_program.into());
            let mut render_state    = self.render_state.lock().unwrap();
            let blob_id             = object.add_blob(&mut render_state.blob_land, self.bounds, Self::blob_interaction_fn(tool_id, &self.objects, &self.tool_id_for_blob_id));

            // Add the render properties for this object (sends it to our render program)
            let property_list       = render_state.object_properties.get();
            let mut property_list   = (*property_list).clone();
            property_list.push(object.render_properties().clone());
            render_state.object_properties.set(Arc::new(property_list));

            self.tool_id_for_blob_id.lock().unwrap().insert(blob_id, tool_id);

            // Start a subprogram to manage this tool
            let tool_id = object.tool().id();
            context.send_message(SceneControl::start_program(object.subprogram_id(),
                move |input, context| physics_object_program(input, context, tool_id),
                0
            )).await.ok();

            self.objects.lock().unwrap().insert(tool_id, object);
        }
    }

    ///
    /// Performs an action on the object with the specified ID
    ///
    pub fn object_action(&mut self, tool_id: PhysicsToolId, action: impl FnOnce(&mut PhysicsObject, &mut BlobLand) -> ()) {
        let objects             = &self.objects;
        let mut render_state    = self.render_state.lock().unwrap();
        let blob_land           = &mut render_state.blob_land;
        let mut objects         = objects.lock().unwrap();

        let maybe_object = objects.get_mut(&tool_id);

        if let Some(object) = maybe_object {
            (action)(object, blob_land)
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
        self.object_action(tool_id, move |object, _| {
            object.set_position(ToolPosition::Float(new_position.0, new_position.1));
        });
    }

    ///
    /// Removes a tool entirely from the state
    ///
    pub fn remove_tool(&mut self, tool_id: PhysicsToolId) {
        if let Some(mut removed_object) = self.objects.lock().unwrap().remove(&tool_id) {
            // Invalidate the sprite and return it to the pool
            if let Some(sprite) = removed_object.invalidate_sprite() {
                self.sprites.push(sprite);
            }

            let blob_id = removed_object.blob_id();
            self.tool_id_for_blob_id.lock().unwrap().remove(&blob_id);
        }
    }

    ///
    /// Invalidates the sprite for a tool, prompting it to be redrawn
    ///
    pub fn invalidate_sprite(&mut self, tool_id: PhysicsToolId) {
        if let Some(object) = self.objects.lock().unwrap().get_mut(&tool_id) {
            // Invalidate the sprite and return it to the pool
            if let Some(sprite) = object.invalidate_sprite() {
                self.sprites.push(sprite);
            }
        }
    }

    ///
    /// Updates the bounds of the window that the tools are contained within
    ///
    pub fn set_bounds(&mut self, width: f64, height: f64) {
        self.bounds = (width, height);
    }

    ///
    /// Updates the focus program on the location of a tool
    ///
    pub async fn update_tool_focus(&mut self, tool_id: PhysicsToolId, focus_events: &mut OutputSink<Focus>) {
        let maybe_focus_event = {
            let mut objects = self.objects.lock().unwrap();

            if let Some(object) = objects.get_mut(&tool_id) {
                // Each object has a control ID
                let program_id = object.subprogram_id();

                if let Some(UiPoint(x, y)) = object.position(self.bounds) {
                    let (w, h)    = object.tool().size();
                    let tool_size = w.max(h);
                    let tool_path = Circle::new(UiPoint(x, y), tool_size/2.0).to_path();

                    // Create a focus region for the tool
                    Some(Focus::ClaimRegion {
                        program:    program_id,
                        region:     vec![tool_path],
                        z_index:    1,
                    })
                } else {
                    // Tool is hidden or otherwise not present
                    Some(Focus::RemoveClaim(program_id))
                }
            } else {
                None
            }
        };

        if let Some(focus_event) = maybe_focus_event {
            focus_events.send(focus_event).await.ok();
        }
    }

    /*
    ///
    /// Runs the blobland simulation and renders the result
    ///
    pub async fn run_simulation(&mut self, drawing_requests: &mut OutputSink<DrawingRequest>) {
        // Read the time since the last tick occurred
        let now             = Instant::now();
        let tick_time       = now.duration_since(self.blob_tick);
        let tick_time_us    = tick_time.as_micros();
        let tick_time_s     = (tick_time_us as f64) / 1_000_000.0;

        // Run a frame of simulation for the blobland
        self.blob_land.simulate(tick_time_s);
        self.blob_tick = now;

        // Render the blobland update
        let mut drawing = vec![];

        drawing.push_state();
        drawing.namespace(*PHYSICS_LAYER);
        drawing.layer(LayerId(0));
        drawing.clear_layer();

        drawing.fill_color(color_tool_background());
        drawing.stroke_color(color_tool_outline());
        drawing.line_width(2.0);
        self.blob_land.render(&mut drawing);

        drawing.pop_state();

        drawing_requests.send(DrawingRequest::Draw(Arc::new(drawing))).await.ok();
    }
    */
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

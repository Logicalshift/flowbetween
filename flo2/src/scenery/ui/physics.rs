//!
//! The physics layer is a part of the UI that uses a physics engine to allow things to be interacted with through
//! physical properties.
//!
//! This is an experimental idea: the physics engine gives the controls a 'feeling', and also stops things like
//! controls overlapping.
//!

use super::blobland::*;
use super::colors::*;
use super::focus::*;
use super::namespaces::*;
use super::physics_object::*;
use super::physics_tool::*;
use super::subprograms::*;
use super::ui_path::*;

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

    /// Timeout indicating that we should run the simulation
    RunSimulation,
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

fn test_object() -> PhysicsTool {
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
    let mut drawing_requests    = context.send::<DrawingRequest>(()).unwrap();
    let mut focus_requests      = context.send::<Focus>(()).unwrap();
    let mut timer_requests      = context.send(()).unwrap();

    // Drawing settings
    let mut state = PhysicsLayerState {
        our_program_id: our_program_id,
        objects:        vec![],
        bounds:         (1024.0, 768.0),
        sprites:        vec![],
        next_sprite_id: 0,
        blob_land:      BlobLand::empty(),
        blob_tick:      Instant::now(),
        blob_awake:     false,
    };

    // Objects on the layer
    let mut objects: Vec<PhysicsObject> = vec![];

    // TEST: create a test object, force an initial update
    let mut test_object = test_object();
    let test_object_id = test_object.id();
    state.add_tool(test_object, StreamTarget::None, &context).await;
    state.float(test_object_id, (100.0, 100.0));
    state.update_tool_focus(test_object_id, &mut focus_requests).await;

    // We're a focus program with only controls, underneath pretty much anything else (so we claim z-index 0)
    focus_requests.send(Focus::ClaimRegion { program: our_program_id, region: vec![], z_index: 0 }).await.ok();

    // Make sure the simulation is awake
    state.wake_simulation(our_program_id, &mut timer_requests).await;

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
                RunSimulation                   => { state.blob_awake = false; state.run_simulation(our_program_id, &mut timer_requests, &mut drawing_requests).await; }

                // Event handling
                Event(FocusEvent::Event(_, DrawEvent::Resize(w, h)))   => { state.set_bounds(w, h); positions_invalidated = true; }

                Event(_draw_event) => { }

                ObjectAction(PhysicsObjectAction::Activate(tool_id))        => { }
                ObjectAction(PhysicsObjectAction::Expand(tool_id))          => { }
                ObjectAction(PhysicsObjectAction::StartDrag(tool_id, x, y)) => { let bounds = state.bounds; state.object_action(tool_id, |object, _| object.start_drag(x, y, bounds)); }
                ObjectAction(PhysicsObjectAction::Drag(tool_id, x, y))      => { state.object_action(tool_id, |object, _| object.drag(x, y)); }
                ObjectAction(PhysicsObjectAction::EndDrag(tool_id, x, y))   => { let bounds = state.bounds; state.object_action(tool_id, |object, blob_land| { object.end_drag(x, y); object.update_blob_position(blob_land, bounds); }); }
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
            // Simulation needs to run/restart if the positions are invalidated
            state.run_simulation(our_program_id, &mut timer_requests, &mut drawing_requests).await;

            let bounds = state.bounds;

            drawing.push_state();
            drawing.namespace(*PHYSICS_LAYER);
            drawing.layer(LayerId(1));
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
    // Program ID of the phyics layer program
    our_program_id: SubProgramId,

    /// Objects in the physics layer
    objects: Vec<PhysicsObject>,

    /// Bounds of the drawing area
    bounds: (f64, f64),

    /// The pool of sprite IDs that have been used by the tools but are now available for other uses
    sprites: Vec<SpriteId>,

    /// The sprite ID that will be assigned if no sprites are available in the pool
    next_sprite_id: u64,

    /// The blobland simulation that causes the tools to animate when they're joined together or pulled apart
    blob_land: BlobLand,

    /// The instant when the last simulation tick was run
    blob_tick: Instant,

    /// True if the simulation is awake
    blob_awake: bool,
}

impl PhysicsLayerState {
    ///
    /// Adds or replaces a tool within this object
    ///
    pub async fn add_tool(&mut self, new_tool: PhysicsTool, target_program: StreamTarget, context: &SceneContext) {
        let existing_idx = self.objects.iter().enumerate()
            .filter(|(_, object)| object.tool().id() == new_tool.id())
            .map(|(idx, _)| idx)
            .next();

        if let Some(existing_idx) = existing_idx {
            // Update the tool in the existing object
            self.objects[existing_idx].set_tool(new_tool, target_program.into());
        } else {
            // Create a new object
            let mut object = PhysicsObject::new(new_tool, target_program.into());
            object.add_blob(&mut self.blob_land, self.bounds);

            // Start a subprogram to manage this tool
            let tool_id = object.tool().id();
            context.send_message(SceneControl::start_program(object.subprogram_id(),
                move |input, context| physics_object_program(input, context, tool_id),
                0
            )).await.ok();

            self.objects.push(object);
        }
    }

    ///
    /// Performs an action on the object with the specified ID
    ///
    pub fn object_action(&mut self, tool_id: PhysicsToolId, action: impl FnOnce(&mut PhysicsObject, &mut BlobLand) -> ()) {
        let existing_idx = self.objects.iter().enumerate()
            .filter(|(_, object)| object.tool().id() == tool_id)
            .map(|(idx, _)| idx)
            .next();

        if let Some(existing_idx) = existing_idx {
            let objects     = &mut self.objects;
            let blob_land   = &mut self.blob_land;

            (action)(&mut objects[existing_idx], blob_land)
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
        let bounds = self.bounds;
        self.object_action(tool_id, move |object, blob_land| {
            object.set_position(ToolPosition::Float(new_position.0, new_position.1));
            object.update_blob_position(blob_land, bounds);
        });
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
        // Find the existing tool
        let existing_idx = self.objects.iter().enumerate()
            .filter(|(_, object)| object.tool().id() == tool_id)
            .map(|(idx, _)| idx)
            .next();

        if let Some(existing_idx) = existing_idx {
            // Each object has a control ID
            let program_id = self.objects[existing_idx].subprogram_id();

            if let Some((x, y)) = self.objects[existing_idx].position(self.bounds) {
                let (w, h)    = self.objects[existing_idx].tool().size();
                let tool_size = w.max(h);
                let tool_path = Circle::new(UiPoint(x, y), tool_size/2.0).to_path();

                // Create a focus region for the tool
                focus_events.send(Focus::ClaimRegion {
                    program:    program_id,
                    region:     vec![tool_path],
                    z_index:    existing_idx,
                }).await.ok();
            } else {
                // Tool is hidden or otherwise not present
                focus_events.send(Focus::RemoveClaim(program_id)).await.ok();
            }
        }
    }

    ///
    /// Runs the blobland simulation
    ///
    pub async fn run_simulation(&mut self, our_program: SubProgramId, timer_requests: &mut OutputSink<TimerRequest>, drawing_requests: &mut OutputSink<DrawingRequest>) {
        // Read the time since the last tick occurred
        let now             = Instant::now();
        let tick_time       = now.duration_since(self.blob_tick);
        let tick_time_us    = tick_time.as_micros();
        let tick_time_s     = (tick_time_us as f64) / 1_000_000.0;

        // Run a frame of simulation for the blobland
        let asleep = self.blob_land.simulate(tick_time_s);
        self.blob_tick = now;

        if !asleep {
            self.wake_simulation(our_program, timer_requests).await;
        }

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

    ///
    /// Causes a request for the simulation to run
    ///
    pub async fn wake_simulation(&mut self, our_program: SubProgramId, timer_requests: &mut OutputSink<TimerRequest>) {
        if !self.blob_awake {
            timer_requests.send(TimerRequest::CallAfter(our_program, 0, Duration::from_micros(1_000_000 / 60))).await.ok();
            self.blob_awake = true;
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
        init_context.connect_programs(StreamSource::Filtered(FilterHandle::for_filter(|timeout_events| timeout_events.map(|_: TimeOut| PhysicsLayer::RunSimulation))), (), StreamId::with_message_type::<TimeOut>()).ok();
    }
}

impl SceneMessage for PhysicsEvent {

}

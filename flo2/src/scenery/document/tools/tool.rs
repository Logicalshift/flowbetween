use crate::scenery::ui::*;

use flo_scene::*;
use flo_draw::*;
use flo_scene::programs::*;

use futures::prelude::*;
use futures::future::{BoxFuture};

use std::collections::*;
use std::sync::*;

///
/// Collection of functions that define the behaviour of a tool
///
pub struct ToolBehaviour<TToolData> {
    /// The name of this tool
    name: String,

    /// Creates the data for the initial tools of this tool class
    create_default_data: Box<dyn Send + Sync + Fn() -> Vec<TToolData>>,

    /// Program that handles events on the canvas while this tool is running
    canvas_program: Box<dyn Send + Sync + Fn(InputStream<FocusEvent>, SceneContext, Arc<Mutex<TToolData>>) -> BoxFuture<'static, ()>>,

    /// Program that deals with updating the tool's main icon
    icon_program: Box<dyn Send + Sync + Fn(InputStream<()>, SceneContext, Arc<Mutex<TToolData>>) -> BoxFuture<'static, ()>>,
}

///
/// Trait implemented by tool data instances
///
pub trait ToolData : Send {
    ///
    /// The initial position for this tool
    ///
    /// The stream target here is the subprogram that owns the initial instance of the tool
    ///
    fn initial_position(&self) -> (StreamTarget, (f64, f64));
}

impl<TToolData> ToolBehaviour<TToolData> 
where 
    TToolData: ToolData,
{
    ///
    /// Creates a tool with no behaviour, and a set of default tools
    ///
    pub fn new(name: impl Into<String>, create_default: impl 'static + Send + Sync + Fn() -> Vec<TToolData>) -> Self {
        Self {
            name:                   name.into(),
            create_default_data:    Box::new(create_default),
            canvas_program:         Box::new(|_, _, _| future::ready(()).boxed()),
            icon_program:           Box::new(|_, _, _| future::ready(()).boxed()),
        }
    }
}

///
/// The subprograms used to run each tool
///
struct ToolSubPrograms {
    /// The canvas subprogram
    canvas_program: Option<OutputSink<FocusEvent>>,

    /// The ID of the canvas program
    canvas_program_id: SubProgramId,

    /// The icon subprogram
    icon_program: Option<OutputSink<()>>,

    /// The ID of the icon program
    icon_program_id: SubProgramId,
}

impl ToolSubPrograms {
    ///
    /// Starts the tool subprograms for a tool data structure
    ///
    async fn start<TToolData>(_tool_id: ToolId, tool_data: &Arc<Mutex<TToolData>>, behaviour: &Arc<ToolBehaviour<TToolData>>, context: &SceneContext) -> Self 
    where 
        TToolData: 'static + ToolData,
    {
        // Assign IDs to the subprograms
        let canvas_program_id   = SubProgramId::new();
        let icon_program_id     = SubProgramId::new();

        // The program running the context acts as the parent program
        let Some(our_program_id) = context.current_program_id() else { return Self { canvas_program: None, icon_program: None, canvas_program_id, icon_program_id } };

        // Start the canvas and icon programs as child programs of the current program
        let canvas_program_behaviour    = Arc::clone(behaviour);
        let canvas_program_data         = Arc::clone(tool_data);
        context.send_message(SceneControl::start_child_program(canvas_program_id, our_program_id, move |input, context| (canvas_program_behaviour.canvas_program)(input, context, canvas_program_data), 1))
            .await.ok();

        let icon_program_behaviour  = Arc::clone(behaviour);
        let icon_program_data       = Arc::clone(tool_data);
        context.send_message(SceneControl::start_child_program(icon_program_id, our_program_id, move |input, context| (icon_program_behaviour.icon_program)(input, context, icon_program_data), 1))
            .await.ok();

        // Connect to the programs we just started
        let canvas_program  = context.send(canvas_program_id).ok();
        let icon_program    = context.send(icon_program_id).ok();

        Self { canvas_program, icon_program, canvas_program_id, icon_program_id }
    }

    ///
    /// Stops the subprograms for this tool
    ///
    async fn stop(&mut self, context: &SceneContext) {
        // Close the input streams for all the tool subprograms
        context.send_message(SceneControl::Close(self.canvas_program_id)).await.ok();
        context.send_message(SceneControl::Close(self.icon_program_id)).await.ok();
    }
}

///
/// Creates a tool subprogram function for a tool
///
pub fn tool_program<TToolData>(tool_type_id: ToolTypeId, group_id: ToolGroupId, behaviour: ToolBehaviour<TToolData>) -> impl 'static + FnOnce(InputStream<ToolState>, SceneContext) -> BoxFuture<'static, ()>
where
    TToolData: 'static + Send + Clone + ToolData,
{
    move |input, context| async move {
        let Some(our_program_id) = context.current_program_id() else { return; };

        // The data for each tool
        let behaviour               = Arc::new(behaviour);
        let mut tool_data           = HashMap::<ToolId, Arc<Mutex<TToolData>>>::new();
        let mut tool_subprograms    = HashMap::<ToolId, ToolSubPrograms>::new();

        // Send messages to the main tool manager for the scene
        let Ok(mut tool_target) = context.send(()) else { return; };

        // Claim this tool type (the parent program acts as the main handler for this tool type)
        tool_target.send(Tool::SetToolTypeOwner(tool_type_id, our_program_id.into())).await.ok();

        // Create the default set of tools
        for default_tool in (behaviour.create_default_data)() {
            // Create the tool ID
            let tool_id = ToolId::new();

            // Store the data for this program
            let default_tool = Arc::new(Mutex::new(default_tool));
            tool_data.insert(tool_id, default_tool.clone());

            // Tell the tool manager about our new tool
            let (default_target, default_location) = default_tool.lock().unwrap().initial_position();
            tool_target.send(Tool::CreateTool(group_id, tool_type_id, tool_id)).await.ok();
            tool_target.send(Tool::SetToolLocation(tool_id, default_target, default_location)).await.ok();

            // Start the subprograms for this tool (these will do things like set the initial icon)
            let subprograms = ToolSubPrograms::start(tool_id, &default_tool, &behaviour, &context).await;
            tool_subprograms.insert(tool_id, subprograms);
        }

        // Handle the main tool events
        let mut input = input;
        while let Some(evt) = input.next().await {
            match evt {
                ToolState::DuplicateTool(from, to)          => { },
                ToolState::Select(_tool_id)                 => { },
                ToolState::OpenDialog(_tool_id)             => { },
                ToolState::CloseDialog(_tool_id)            => { },
                ToolState::Deselect(_tool_id)               => { },

                ToolState::RemoveTool(tool_id)              => {
                    // Remove the tool data
                    tool_data.remove(&tool_id);
                    let Some(mut subprograms) = tool_subprograms.remove(&tool_id) else { continue; };

                    // Stop the subprograms associated with this tool
                    subprograms.stop(&context).await;
                },

                ToolState::AddTool(_tool_id)                => { },
                ToolState::LocateTool(_tool_id, _)          => { },
                ToolState::SetName(_tool_id, _)             => { },
                ToolState::SetIcon(_tool_id, _)             => { },
                ToolState::SetDialogLocation(_tool_id, _)   => { },
            }
        }

        // Send messages to remove all the tools once this program is done
        for (tool_id, subprograms) in tool_subprograms.drain() {
            // Remove the tool from the manager
            tool_target.send(Tool::RemoveTool(tool_id)).await.ok();

            // Tell the subprograms to stop
            let mut subprograms = subprograms;
            subprograms.stop(&context).await;
        }
    }.boxed()
}

use crate::scenery::ui::*;

use flo_scene::*;
use flo_draw::*;
use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;

///
/// Collection of functions that define the behaviour of a tool
///
pub struct ToolBehaviour<TToolData> {
    /// The name of this tool
    name: String,

    /// Creates the data for the initial tool
    create_default_data: Box<dyn Send + Fn() -> TToolData>,

    /// The location where the initial tool is created
    initial_location: Option<(StreamTarget, (f64, f64))>,

    /// Program that handles events on the canvas while this tool is running
    canvas_program: Box<dyn Send + Fn(InputStream<FocusEvent>, SceneContext, Arc<Mutex<TToolData>>) -> BoxFuture<'static, ()>>,

    /// Program that deals with updating the tool's main icon
    icon_program: Box<dyn Send + Fn(InputStream<()>, SceneContext, Arc<Mutex<TToolData>>) -> BoxFuture<'static, ()>>,
}

impl<TToolData> ToolBehaviour<TToolData> {
    ///
    /// Creates a tool with no behaviour
    ///
    pub fn new(name: impl Into<String>, create_default: impl 'static + Send + Fn() -> TToolData) -> Self {
        Self {
            name:                   name.into(),
            create_default_data:    Box::new(create_default),
            initial_location:       None,
            canvas_program:         Box::new(|_, _, _| future::ready(()).boxed()),
            icon_program:           Box::new(|_, _, _| future::ready(()).boxed()),
        }
    }
}

///
/// Creates a tool subprogram function for a tool
///
pub fn tool_program<TToolData>(tool_type: ToolTypeId, group_id: ToolGroupId, behaviour: ToolBehaviour<TToolData>) -> impl 'static + FnOnce(InputStream<ToolState>, SceneContext) -> BoxFuture<'static, ()>
where
    TToolData: Send + Clone,
{
    move |input, context| async move {
        let Some(our_program_id) = context.current_program_id() else { return; };

        // Register the tool
        let Ok(mut tool_target) = context.send(()) else { return; };

        // Claim this tool type
        tool_target.send(Tool::SetToolTypeOwner(tool_type, our_program_id.into())).await.ok();

        // Create the default tool ID
        let default_tool_id = ToolId::new();

        tool_target.send(Tool::CreateTool(group_id, tool_type, default_tool_id)).await.ok();
        if let Some((default_target, default_location)) = behaviour.initial_location {
            tool_target.send(Tool::SetToolLocation(default_tool_id, default_target, default_location)).await.ok();
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

                ToolState::AddTool(_tool_id)                => { },
                ToolState::RemoveTool(_tool_id)             => { },
                ToolState::LocateTool(_tool_id, _)          => { },
                ToolState::SetName(_tool_id, _)             => { },
                ToolState::SetIcon(_tool_id, _)             => { },
                ToolState::SetDialogLocation(_tool_id, _)   => { },
            }
        }
    }.boxed()
}

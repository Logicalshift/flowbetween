use crate::scenery::ui::*;

use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;

///
/// Collection of functions that define the behaviour of a tool
///
pub struct ToolBehaviour<TToolData> {
    /// Program that handles events on the canvas while this tool is running
    canvas_program: Box<dyn Fn(InputStream<DrawEvent>, SceneContext, Arc<Mutex<TToolData>>) -> BoxFuture<'static, ()>>,
}

///
/// Creates a tool subprogram function for a tool
///
pub fn tool_program<TToolData>(tool_type: ToolTypeId, group_id: ToolGroupId, behaviour: ToolBehaviour) -> impl 'static + FnOnce(InputStream<ToolState>, SceneContext) -> BoxFuture<'static, ()>
where
    TToolData: Send + Clone,
{

}

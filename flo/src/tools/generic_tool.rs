use super::tool_trait::*;
use super::tool_input::*;
use super::tool_action::*;
use super::super::viewmodel::*;

use animation::*;

use futures::*;

use std::any::*;
use std::sync::*;
use std::marker::PhantomData;

///
/// The generic tool is used to convert a tool that uses a specific data type
/// to one that uses a standard data type. This makes it possible to use tools
/// without needing to know their underlying implementation.
/// 
/// A generic tool is typically used as its underlying Tool trait, for example
/// in an `Arc` reference.
/// 
pub struct GenericTool<ToolData: Send+'static, Anim: Animation, UnderlyingTool: Tool2<ToolData, Anim>> {
    /// The tool that this makes generic
    tool: UnderlyingTool,

    // Phantom data for the tool trait parameters
    phantom_anim: PhantomData<Anim>,
    phantom_tooldata: PhantomData<ToolData>
}

///
/// The data structure storing the generic tool data
/// 
pub struct GenericToolData(Box<Any+Send>);

///
/// Converts a tool to a generic tool
/// 
pub trait ToGenericTool<Anim: Animation, ToolData: Send+'static> {
    ///
    /// Converts this object to a generic tool reference
    /// 
    fn to_generic_tool(self) -> Arc<Tool2<GenericToolData, Anim>>;
}

impl GenericToolData {
    ///
    /// Converts an action to generic tool data
    /// 
    fn convert_action_to_generic<ToolData: 'static+Send>(action: ToolAction<ToolData>) -> ToolAction<GenericToolData> {
        use self::ToolAction::*;

        match action {
            Data(data)              => Data(GenericToolData(Box::new(data))),
            Edit(edit)              => Edit(edit),
            BrushPreview(preview)   => BrushPreview(preview)
        }
    }

    ///
    /// Converts to a refernece of the specified type if possible
    /// 
    fn convert_ref<ToolData: 'static+Send>(&self) -> Option<&ToolData> {
        self.0.downcast_ref()
    }

    ///
    /// Converts an input value from generic tool data  to specific tool data
    /// 
    fn convert_input_from_generic<'a, ToolData: 'static+Send>(input: ToolInput<'a, GenericToolData>) -> Option<ToolInput<'a, ToolData>> {
        use self::ToolInput::*;

        match input {
            Data(ref data)      => data.convert_ref().map(|data| Data(data)),
            PaintDevice(device) => Some(PaintDevice(device)),
            Paint(paint)        => Some(Paint(paint))
        }
    }
}

impl<ToolData: Send+'static, Anim: Animation, UnderlyingTool: Tool2<ToolData, Anim>> From<UnderlyingTool> for GenericTool<ToolData, Anim, UnderlyingTool> {
    fn from(tool: UnderlyingTool) -> GenericTool<ToolData, Anim, UnderlyingTool> {
        GenericTool {
            tool:               tool,
            phantom_anim:       PhantomData,
            phantom_tooldata:   PhantomData
        }
    }
}

impl<ToolData: Send+'static, Anim: Animation, UnderlyingTool: Tool2<ToolData, Anim>> Tool2<GenericToolData, Anim> for GenericTool<ToolData, Anim, UnderlyingTool> {
    fn tool_name(&self) -> String {
        self.tool.tool_name()
    }

    fn image_name(&self) -> String {
        self.tool.image_name()        
    }

    fn menu_controller_name(&self) -> String {
        self.tool.menu_controller_name()
    }

    fn actions_for_model(&self, model: Arc<AnimationViewModel<Anim>>) -> Box<Stream<Item=ToolAction<GenericToolData>, Error=()>> {
        // Map the underlying actions to generic actions
        Box::new(self.tool.actions_for_model(model)
            .map(|action| GenericToolData::convert_action_to_generic(action)))
    }

    fn actions_for_input<'b>(&self, data: Option<&'b GenericToolData>, input: Box<'b+Iterator<Item=ToolInput<'b, GenericToolData>>>) -> Box<'b+Iterator<Item=ToolAction<GenericToolData>>> {
        // Generic data items from other tools don't generate data for this tool
        let data    = data.and_then(|data| data.convert_ref());
        let input   = Box::new(input
            .map(|input_item|       GenericToolData::convert_input_from_generic(input_item))
            .filter(|maybe_data|    maybe_data.is_some())
            .map(|definitely_data|  definitely_data.unwrap()));

        // Map the actions back to generic actions
        Box::new(self.tool.actions_for_input(data, input)
            .map(|action| GenericToolData::convert_action_to_generic(action)))
    }
}

impl<Anim: 'static+Animation, ToolData: 'static+Send, T: 'static+Tool2<ToolData, Anim>> ToGenericTool<Anim, ToolData> for T {
    fn to_generic_tool(self) -> Arc<Tool2<GenericToolData, Anim>> {
        Arc::new(GenericTool::from(self))
    }
}
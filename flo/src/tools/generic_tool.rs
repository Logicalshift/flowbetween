use super::tool_trait::*;
use super::tool_input::*;
use super::tool_action::*;
use super::super::model::*;

use animation::*;

use futures::*;

use std::fmt;
use std::any::*;
use std::sync::*;
use std::marker::PhantomData;

///
/// Trait implemented by FlowBetween tools
/// 
/// FloTools eliminate the need to know what the tool data structure stores.
/// 
pub trait FloTool<Anim: Animation> : Tool<GenericToolData, Anim> {
}

///
/// The generic tool is used to convert a tool that uses a specific data type
/// to one that uses a standard data type. This makes it possible to use tools
/// without needing to know their underlying implementation.
/// 
/// A generic tool is typically used as its underlying Tool trait, for example
/// in an `Arc` reference.
/// 
pub struct GenericTool<ToolData: Send+'static, Anim: Animation, UnderlyingTool: Tool<ToolData, Anim>> {
    /// The tool that this makes generic
    tool: UnderlyingTool,

    // Phantom data for the tool trait parameters
    phantom_anim: PhantomData<Anim>,
    phantom_tooldata: PhantomData<Mutex<ToolData>>
}

///
/// The data structure storing the generic tool data
/// 
pub struct GenericToolData(Mutex<Box<Any+Send>>);

impl fmt::Debug for GenericToolData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_str("GenericToolData")
    }
}

///
/// Converts a tool to a generic tool
/// 
pub trait ToFloTool<Anim: Animation, ToolData: Send+'static> {
    ///
    /// Converts this object to a generic tool reference
    /// 
    fn to_flo_tool(self) -> Arc<FloTool<Anim>>;
}

impl GenericToolData {
    ///
    /// Converts an action to generic tool data
    /// 
    fn convert_action_to_generic<ToolData: 'static+Send+Sync>(action: ToolAction<ToolData>) -> ToolAction<GenericToolData> {
        use self::ToolAction::*;

        match action {
            Data(data)              => Data(GenericToolData(Mutex::new(Box::new(Arc::new(data))))),
            Edit(edit)              => Edit(edit),
            BrushPreview(preview)   => BrushPreview(preview)
        }
    }

    ///
    /// Converts to a refernece of the specified type if possible
    /// 
    fn convert_ref<ToolData: 'static+Send>(&self) -> Option<Arc<ToolData>> {
        self.0.lock().unwrap().downcast_ref().cloned()
    }

    ///
    /// Converts an input value from generic tool data  to specific tool data
    /// 
    fn convert_input_from_generic<ToolData: 'static+Send>(input: ToolInput<GenericToolData>) -> Option<ToolInput<ToolData>> {
        use self::ToolInput::*;

        match input {
            Data(ref data)      => data.convert_ref().map(|data| Data(data)),
            PaintDevice(device) => Some(PaintDevice(device)),
            Paint(paint)        => Some(Paint(paint))
        }
    }
}

impl<ToolData: Send+'static, Anim: Animation, UnderlyingTool: Tool<ToolData, Anim>> From<UnderlyingTool> for GenericTool<ToolData, Anim, UnderlyingTool> {
    fn from(tool: UnderlyingTool) -> GenericTool<ToolData, Anim, UnderlyingTool> {
        GenericTool {
            tool:               tool,
            phantom_anim:       PhantomData,
            phantom_tooldata:   PhantomData
        }
    }
}

impl<ToolData: Send+Sync+'static, Anim: Animation, UnderlyingTool: Tool<ToolData, Anim>> Tool<GenericToolData, Anim> for GenericTool<ToolData, Anim, UnderlyingTool> {
    fn tool_name(&self) -> String {
        self.tool.tool_name()
    }

    fn image_name(&self) -> String {
        self.tool.image_name()        
    }

    fn menu_controller_name(&self) -> String {
        self.tool.menu_controller_name()
    }

    fn actions_for_model(&self, model: Arc<FloModel<Anim>>) -> Box<Stream<Item=ToolAction<GenericToolData>, Error=()>+Send> {
        // Map the underlying actions to generic actions
        Box::new(self.tool.actions_for_model(model)
            .map(|action| GenericToolData::convert_action_to_generic(action)))
    }

    fn actions_for_input<'a>(&'a self, data: Option<Arc<GenericToolData>>, input: Box<'a+Iterator<Item=ToolInput<GenericToolData>>>) -> Box<'a+Iterator<Item=ToolAction<GenericToolData>>> {
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

impl<ToolData: Send+Sync+'static, Anim: Animation, UnderlyingTool: Tool<ToolData, Anim>> FloTool<Anim> for GenericTool<ToolData, Anim, UnderlyingTool> {
}

///
/// Converts any tool to its generic 'FloTool' equivalent
/// 
impl<Anim: 'static+Animation, ToolData: 'static+Send+Sync, T: 'static+Tool<ToolData, Anim>> ToFloTool<Anim, ToolData> for T {
    fn to_flo_tool(self) -> Arc<FloTool<Anim>> {
        Arc::new(GenericTool::from(self))
    }
}

///
/// Equality so that tool objects can be referred to in bindings
/// 
impl<Anim: Animation> PartialEq for FloTool<Anim> {
    fn eq(&self, other: &FloTool<Anim>) -> bool {
        self.tool_name() == other.tool_name()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::brush_preview_action::*;
    use animation::inmemory::*;

    struct TestTool;

    impl Tool<i32, InMemoryAnimation> for TestTool {
        fn tool_name(&self) -> String { "test".to_string() }

        fn image_name(&self) -> String { "test".to_string() }

        fn actions_for_input<'a>(&'a self, data: Option<Arc<i32>>, input: Box<'a+Iterator<Item=ToolInput<i32>>>) -> Box<'a+Iterator<Item=ToolAction<i32>>> {
            let input: Vec<ToolInput<i32>> = input.collect();
            
            if input.len() == 1 {
                match &input[0] {
                    &ToolInput::Data(ref data) => {
                        if **data == 42 {
                            // Signals to the test that the data made the round trip
                            Box::new(vec![ToolAction::BrushPreview(BrushPreviewAction::Clear)].into_iter())
                        } else {
                            // Data is incorrect
                            Box::new(vec![].into_iter())
                        }
                    }

                    // Action is incorrect
                    _ => Box::new(vec![].into_iter())
                }

            } else {
                // No actions
                Box::new(vec![ToolAction::Data(42)].into_iter())
            }
        }
    }

    #[test]
    fn generates_generic_data_for_standard_data() {
        let generic_tool = TestTool.to_flo_tool();

        let actions = generic_tool.actions_for_input(None, Box::new(vec![].into_iter()));
        let actions: Vec<ToolAction<GenericToolData>> = actions.collect();

        assert!(actions.len() == 1);
        assert!(match &actions[0] {
            &ToolAction::Data(_) => true,
            _ => false
        });
    }

    #[test]
    fn data_survives_round_trip() {
        let generic_tool = TestTool.to_flo_tool();

        let actions = generic_tool.actions_for_input(None, Box::new(vec![].into_iter()));
        let mut actions: Vec<ToolAction<GenericToolData>> = actions.collect();

        // Should return a data element of '42'
        let data = match actions.pop() {
            Some(ToolAction::Data(data)) => Some(Arc::new(data)),
            _ => None
        }.unwrap();

        // Feed this back into the tool: should generate a 'clear' brush preview action as a result (see tool definition)
        let feedback_actions = generic_tool.actions_for_input(None, Box::new(vec![ToolInput::Data(data)].into_iter()));
        let feedback_actions: Vec<ToolAction<GenericToolData>> = feedback_actions.collect();

        assert!(feedback_actions.len() == 1);
        assert!(match &feedback_actions[0] {
            &ToolAction::BrushPreview(BrushPreviewAction::Clear) => true,
            _ => false
        });
    }
}

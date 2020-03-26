use super::tool_trait::*;
use super::tool_input::*;
use super::tool_action::*;
use super::super::model::*;

use flo_ui::*;
use flo_animation::*;

use futures::*;
use futures::stream;
use futures::stream::{BoxStream};

use std::fmt;
use std::any::*;
use std::sync::*;
use std::marker::PhantomData;

///
/// Trait implemented by FlowBetween tools
///
/// FloTools eliminate the need to know what the tool data structure stores.
///
pub type FloTool<Anim> = dyn Tool<Anim, ToolData=GenericToolData, Model=GenericToolModel>;

///
/// The generic tool is used to convert a tool that uses a specific data type
/// to one that uses a standard data type. This makes it possible to use tools
/// without needing to know their underlying implementation.
///
/// A generic tool is typically used as its underlying Tool trait, for example
/// in an `Arc` reference.
///
pub struct GenericTool<ToolData: Send+'static, Model: Send+Sync+'static, Anim: Animation, UnderlyingTool: Tool<Anim, ToolData=ToolData, Model=Model>> {
    /// The tool that this makes generic
    tool: UnderlyingTool,

    // Phantom data for the tool trait parameters
    phantom_animation:  PhantomData<Anim>,
    phantom_tooldata:   PhantomData<Mutex<ToolData>>,
    phantom_model:      PhantomData<Model>
}

///
/// The data structure storing the generic tool data
///
pub struct GenericToolData(Mutex<Box<dyn Any+Send>>);

///
/// The data structure storing the generic tool model
///
pub struct GenericToolModel(Mutex<Box<dyn Any+Send>>);

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
            BrushPreview(preview)   => BrushPreview(preview),
            Overlay(overlay)        => Overlay(overlay),
            Select(element)         => Select(element),
            ClearSelection          => ClearSelection,
            InvalidateFrame         => InvalidateFrame
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
            Select              => Some(Select),
            Deselect            => Some(Deselect),
            Data(ref data)      => data.convert_ref().map(|data| Data(data)),
            PaintDevice(device) => Some(PaintDevice(device)),
            Paint(paint)        => Some(Paint(paint))
        }
    }
}

impl GenericToolModel {
    ///
    /// Retrieves a reference to the tool model
    ///
    fn get_ref<Model: 'static+Send>(&self) -> Option<Arc<Model>> {
        self.0.lock().unwrap().downcast_ref().cloned()
    }
}

impl<ToolData: Send+'static, Model: Send+Sync+'static, Anim: Animation, UnderlyingTool: Tool<Anim, ToolData=ToolData, Model=Model>> From<UnderlyingTool> for GenericTool<ToolData, Model, Anim, UnderlyingTool> {
    fn from(tool: UnderlyingTool) -> GenericTool<ToolData, Model, Anim, UnderlyingTool> {
        GenericTool {
            tool:               tool,
            phantom_animation:  PhantomData,
            phantom_tooldata:   PhantomData,
            phantom_model:      PhantomData
        }
    }
}

impl<ToolData: Send+Sync+'static, Model: Send+Sync+'static, Anim: Animation, UnderlyingTool: Tool<Anim, ToolData=ToolData, Model=Model>> Tool<Anim> for GenericTool<ToolData, Model, Anim, UnderlyingTool> {
    type ToolData   = GenericToolData;
    type Model      = GenericToolModel;

    fn tool_name(&self) -> String {
        self.tool.tool_name()
    }

    fn image_name(&self) -> String {
        self.tool.image_name()
    }

    fn create_model(&self, flo_model: Arc<FloModel<Anim>>) -> GenericToolModel {
        GenericToolModel(Mutex::new(Box::new(Arc::new(self.tool.create_model(flo_model)))))
    }

    fn create_menu_controller(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &GenericToolModel) -> Option<Arc<dyn Controller>> {
        tool_model
            .get_ref()
            .and_then(move |specific_model| self.tool.create_menu_controller(flo_model, &*specific_model))
    }

    fn actions_for_model(&self, flo_model: Arc<FloModel<Anim>>, tool_model: &GenericToolModel) -> BoxStream<'static, ToolAction<GenericToolData>> {
        // Map the underlying actions to generic actions. There are no actions if we're passed an invalid tool model
        tool_model.get_ref()
            .map(move |tool_model| -> BoxStream<'static, ToolAction<GenericToolData>> {
                Box::pin(self.tool.actions_for_model(flo_model, &*tool_model)
                    .map(|action| GenericToolData::convert_action_to_generic(action)))
            })
            .unwrap_or_else(|| Box::pin(stream::empty()))
    }

    fn actions_for_input<'a>(&'a self, flo_model: Arc<FloModel<Anim>>, data: Option<Arc<GenericToolData>>, input: Box<dyn 'a+Iterator<Item=ToolInput<GenericToolData>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<GenericToolData>>> {
        // Generic data items from other tools don't generate data for this tool
        let data    = data.and_then(|data| data.convert_ref());
        let input   = Box::new(input
            .map(|input_item|       GenericToolData::convert_input_from_generic(input_item))
            .filter(|maybe_data|    maybe_data.is_some())
            .map(|definitely_data|  definitely_data.unwrap()));

        // Map the actions back to generic actions
        Box::new(self.tool.actions_for_input(flo_model, data, input)
            .map(|action| GenericToolData::convert_action_to_generic(action)))
    }
}

///
/// Converts any tool to its generic 'FloTool' equivalent
///
impl<Anim: 'static+Animation, ToolData: 'static+Send+Sync, T: 'static+Tool<Anim, ToolData=ToolData>> ToFloTool<Anim, ToolData> for T {
    fn to_flo_tool(self) -> Arc<FloTool<Anim>> {
        Arc::new(GenericTool::from(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::brush_preview_action::*;
    use flo_animation::storage::*;

    struct TestTool;

    impl<Anim: 'static+EditableAnimation> Tool<Anim> for TestTool {
        type ToolData   = i32;
        type Model      = i32;

        fn tool_name(&self) -> String { "test".to_string() }

        fn image_name(&self) -> String { "test".to_string() }

        fn create_model(&self, _flo_model: Arc<FloModel<Anim>>) -> i32 { 94 }

        fn actions_for_input<'a>(&'a self, _flo_model: Arc<FloModel<Anim>>, _data: Option<Arc<i32>>, input: Box<dyn 'a+Iterator<Item=ToolInput<i32>>>) -> Box<dyn 'a+Iterator<Item=ToolAction<i32>>> {
            let input: Vec<_> = input.collect();

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
        let generic_tool    = TestTool.to_flo_tool();

        let in_memory_store = InMemoryStorage::new();
        let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());
        let animation       = Arc::new(FloModel::new(animation));
        let actions         = generic_tool.actions_for_input(animation, None, Box::new(vec![].into_iter()));
        let actions: Vec<_> = actions.collect();

        assert!(actions.len() == 1);
        assert!(match &actions[0] {
            &ToolAction::Data(_) => true,
            _ => false
        });
    }

    #[test]
    fn data_survives_round_trip() {
        let generic_tool        = TestTool.to_flo_tool();

        let in_memory_store     = InMemoryStorage::new();
        let animation           = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());
        let animation           = Arc::new(FloModel::new(animation));
        let actions             = generic_tool.actions_for_input(animation.clone(), None, Box::new(vec![].into_iter()));
        let mut actions: Vec<_> = actions.collect();

        // Should return a data element of '42'
        let data = match actions.pop() {
            Some(ToolAction::Data(data)) => Some(Arc::new(data)),
            _ => None
        }.unwrap();

        // Feed this back into the tool: should generate a 'clear' brush preview action as a result (see tool definition)
        let feedback_actions            = generic_tool.actions_for_input(animation.clone(), None, Box::new(vec![ToolInput::Data(data)].into_iter()));
        let feedback_actions: Vec<_>    = feedback_actions.collect();

        assert!(feedback_actions.len() == 1);
        assert!(match &feedback_actions[0] {
            &ToolAction::BrushPreview(BrushPreviewAction::Clear) => true,
            _ => false
        });
    }

    #[test]
    fn model_survives_round_trip() {
        let in_memory_store = InMemoryStorage::new();
        let animation       = create_animation_editor(move |commands| in_memory_store.get_responses(commands).boxed());
        let flo_model       = Arc::new(FloModel::new(animation));
        let generic_tool    = TestTool.to_flo_tool();
        let model           = generic_tool.create_model(Arc::clone(&flo_model));

        assert!(model.get_ref() == Some(Arc::new(94)));
    }
}

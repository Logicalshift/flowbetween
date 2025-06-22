use uuid::*;
use ::serde::*;

use flo_draw::canvas::*;
use flo_binding::*;

///
/// Identifier used to specify a physics tool within the flowbetween app
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysicsToolId(Uuid);

impl PhysicsToolId {
    ///
    /// Creates a unique new physics tool ID
    ///
    pub fn new() -> Self {
        PhysicsToolId(Uuid::new_v4())
    }
}

///
/// Identifier used to specify a tool group within the flowbetween app
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolGroupId(Uuid);

impl ToolGroupId {
    ///
    /// Creates a unique new tool group ID
    ///
    pub fn new() -> Self {
        ToolGroupId(Uuid::new_v4())
    }
}

///
/// A physics tool is a selectable tool, whose UI follows the 'physics' rules
///
pub struct PhysicsTool {
    /// Unique identifier for this tool
    id: PhysicsToolId,

    /// The used to display this tool in the UI
    icon: BindRef<Vec<Draw>>,

    /// The name of this tool
    name: BindRef<String>,

    /// This tool can be selected exclusively amongst this group
    selection_group: Binding<ToolGroupId>,

    /// This tool can be bound to any number of tools within this group
    bind_with: Binding<Vec<ToolGroupId>>,
}

impl PhysicsTool {
    ///
    /// Creates a new physics tool
    ///
    pub fn new(id: PhysicsToolId) -> PhysicsTool {
        PhysicsTool {
            id:                 id,
            icon:               BindRef::from(&Binding::new(vec![])),
            name:               BindRef::from(&Binding::new(String::default())),
            selection_group:    Binding::new(ToolGroupId::new()),
            bind_with:          Binding::new(vec![]),
        }
    }

    ///
    /// Sets the icon to use for this tool
    ///
    pub fn with_icon(mut self, icon: impl Into<Vec<Draw>>) -> Self {
        self.icon = BindRef::from(&Binding::new(icon.into()));

        self
    }

    ///
    /// Sets the icon to use for this tool
    ///
    pub fn with_icon_binding<TBound>(mut self, icon: TBound) -> Self 
    where
        BindRef<Vec<Draw>>: From<TBound>,
    {
        self.icon = BindRef::from(icon);

        self
    }


    ///
    /// Sets the name for this tool
    ///
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = BindRef::from(&Binding::new(name.into()));

        self
    }

    ///
    /// Sets the name for this tool
    ///
    pub fn with_name_binding<TBound>(mut self, name: TBound) -> Self 
    where
        BindRef<String>: From<TBound>,
    {
        self.name = BindRef::from(name);

        self
    }

    ///
    /// Sets the selection group for this tool
    ///
    pub fn with_selection_group(mut self, group: ToolGroupId) -> Self {
        self.selection_group.set(group);
        self
    }

    ///
    /// Sets the tools that this can be bound with
    ///
    pub fn with_bind_with(mut self, bind_with: Vec<ToolGroupId>) -> Self {
        self.bind_with.set(bind_with);
        self
    }

    ///
    /// The icon to display to represent this tool
    ///
    pub fn icon(&self) -> Vec<Draw> {
        self.icon.get()
    }

    ///
    /// The name for this tool
    ///
    pub fn name(&self) -> String {
        self.name.get()
    }

    ///
    /// The selection group this tool is in. Other tools in this same group will be deselected if this tool is chosen
    ///
    pub fn selection_group(&self) -> ToolGroupId {
        self.selection_group.get()
    }

    ///
    /// The tool groups that can be 'bound' to this tool (eg, properties like colour)
    ///
    pub fn bind_with(&self) -> Vec<ToolGroupId> {
        self.bind_with()
    }
}

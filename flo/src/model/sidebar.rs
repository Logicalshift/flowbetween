use crate::sidebar::panel::*;

use flo_binding::*;

///
/// The possible states the sidebar can be in
///
/// The sidebar is mostly used for editing selections, and is generally best hidden while drawing.
/// However, sometimes it's useful even when using a different tool (for example, for editing the
/// document settings).
///
/// The sidebar's open state is thus dependent on what state it was in when the user last closed 
/// it. If the sidebar is closed while 'inactive', then it will automatically open again when it
/// becomes 'active'. Similarly, if opened while 'active' it will automatically close when it
/// becomes 'inactive'.
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SidebarOpenState {
    /// The sidebar is open regardless of whether or not it is considered to be 'active'
    AlwaysOpen,

    /// The sidebar is opened whenever it is 
    OpenWhenActive,

    /// The sidebar is kept closed
    AlwaysClosed,
}

///
/// The possible 'activation' states of the sidebar
///
/// The sidebar is considered 'active' when the user makes a selection or performs another action
/// that has specific settings available in the sidebar.
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SidebarActivationState {
    /// Ths sidebar is only showing generic document-level items
    Inactive,

    /// There are settings relative to the user's current context available in the sidebar
    Active,
}

impl SidebarOpenState {
    ///
    /// Returns whether or not a given activation state should show the sidebar as open or closed
    ///
    pub fn is_open(&self, activation_state: &SidebarActivationState) -> bool {
        match (self, activation_state) {
            (SidebarOpenState::AlwaysOpen, _)                                       => true,
            (SidebarOpenState::OpenWhenActive, SidebarActivationState::Active)      => true,
            (SidebarOpenState::OpenWhenActive, SidebarActivationState::Inactive)    => false,
            (SidebarOpenState::AlwaysClosed, _)                                     => false
        }
    }
}

///
/// Model representing the state of the sidebar controller
///
#[derive(Clone)]
pub struct SidebarModel {
    /// Whether or not the sidebar has been opened by the user
    pub open_state: Binding<SidebarOpenState>,

    /// List of identifiers in priority order of the sidebar items that are open (hidden sidebars can be specified as open, sidebars are collapsed in priority order when they won't all fit on screen)
    pub open_sidebars: Binding<Vec<String>>,

    /// The activation state of the sidebar
    pub activation_state: BindRef<SidebarActivationState>,

    /// The panels contained within this sidebar
    pub panels: RopeBinding<SidebarPanel, ()>,

    /// The panels relating to the document
    document_panels: RopeBindingMut<SidebarPanel, ()>,

    /// The panels relating to the current selection
    selection_panels: RopeBindingMut<SidebarPanel, ()>,

    /// The panels relating to the currently selected tool
    tool_panels: RopeBindingMut<SidebarPanel, ()>
}

impl SidebarModel {
    ///
    /// Creates a new model for the sidebar panel used in FlowBetween
    ///
    pub fn new() -> SidebarModel {
        // Create the default set of panels
        let document_panels     = RopeBindingMut::new();
        let selection_panels    = RopeBindingMut::new();
        let tool_panels         = RopeBindingMut::new();

        // Combine the panels into a single list
        let panels              = document_panels.chain(&selection_panels).chain(&tool_panels);

        // Set up the activation state
        let activation_state    = bind(SidebarActivationState::Inactive);
        let activation_state    = BindRef::from(activation_state);

        SidebarModel {
            open_state:         bind(SidebarOpenState::OpenWhenActive),
            open_sidebars:      bind(vec![]),
            activation_state:   activation_state,
            panels:             panels,
            document_panels:    document_panels,
            selection_panels:   selection_panels,
            tool_panels:        tool_panels
        }
    }
}

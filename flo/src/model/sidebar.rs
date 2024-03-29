use crate::sidebar::panel::*;

use futures::prelude::*;
use futures::stream;

use ::desync::*;
use flo_rope::*;
use flo_stream::*;
use flo_binding::*;

use std::sync::*;

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
    document_panels: RopeBinding<SidebarPanel, ()>,

    /// The panels relating to the current selection
    selection_panels: RopeBinding<SidebarPanel, ()>,

    /// Switches the stream for the selection panels
    selection_panel_switch: Arc<StreamSwitch<RopeAction<SidebarPanel, ()>>>,

    /// Switches the stream for the document panels
    document_panel_switch: Arc<StreamSwitch<RopeAction<SidebarPanel, ()>>>,

    /// The panels relating to the currently selected tool
    tool_panels: RopeBindingMut<SidebarPanel, ()>,

    /// Task that opens new active panels
    open_active_panels: Arc<Desync<()>>
}

impl SidebarModel {
    ///
    /// Creates a new model for the sidebar panel used in FlowBetween
    ///
    pub fn new() -> SidebarModel {
        // Create the default set of panels
        let (panel_stream, document_switch)     = switchable_stream(stream::empty());
        let document_panels                     = RopeBinding::from_stream(panel_stream);
        let (panel_stream, selection_switch)    = switchable_stream(stream::empty());
        let selection_panels                    = RopeBinding::from_stream(panel_stream);
        let tool_panels                         = RopeBindingMut::new();

        // Combine the panels into a single list
        let panels              = selection_panels.chain(&document_panels).chain(&tool_panels);
        let open_sidebars       = bind(vec![]);
        let open_active_panels  = Self::open_new_active_panels(&panels, &open_sidebars);

        // Set up the activation state
        let activation_state    = Self::activation_state(&panels);

        SidebarModel {
            open_state:             bind(SidebarOpenState::OpenWhenActive),
            open_sidebars:          open_sidebars,
            activation_state:       activation_state,
            panels:                 panels,
            document_panels:        document_panels,
            selection_panels:       selection_panels,
            selection_panel_switch: Arc::new(selection_switch),
            document_panel_switch:  Arc::new(document_switch),
            tool_panels:            tool_panels,
            open_active_panels:     open_active_panels
        }
    }

    ///
    /// Updates the 'document-wide' panels used for the sidebar
    ///
    pub fn set_document_panels(&self, new_panels: impl 'static+Send+Stream<Item=RopeAction<SidebarPanel, ()>>) {
        // Switch to an empty stream temporarily
        self.document_panel_switch.switch_to_stream(stream::empty());

        // Clear the panel rope before receiving the new panel stream (which is assumed to be against an empty rope)
        let len         = self.document_panels.len();
        let new_panels  = stream::iter(vec![RopeAction::Replace(0..len, vec![])]).chain(new_panels);

        // Use the new stream
        self.document_panel_switch.switch_to_stream(new_panels);
    }

    ///
    /// Sets the sidebar panels related to the current selection
    ///
    pub fn set_selection_panels(&self, new_panels: impl 'static+Send+Stream<Item=RopeAction<SidebarPanel, ()>>) {
        // Switch to an empty stream temporarily
        self.selection_panel_switch.switch_to_stream(stream::empty());

        // Clear the panel rope before receiving the new panel stream (which is assumed to be against an empty rope)
        let len         = self.selection_panels.len();
        let new_panels  = stream::iter(vec![RopeAction::Replace(0..len, vec![])]).chain(new_panels);

        // Use the new stream
        self.selection_panel_switch.switch_to_stream(new_panels);
    }

    ///
    /// Updates the sidebar panels relating to the currently selected tool
    ///
    pub fn set_tool_panels(&self, new_panels: Vec<SidebarPanel>) {
        self.tool_panels.replace(0..self.tool_panels.len(), new_panels);
    }

    ///
    /// Follows the changes to the panels and opens any new panels that are active
    ///
    fn open_new_active_panels(panels: &RopeBinding<SidebarPanel, ()>, open_sidebars: &Binding<Vec<String>>) -> Arc<Desync<()>> {
        // Create a desync that we'll use to follow the stream
        let runner          = Arc::new(Desync::new(()));

        // Follow the changes to the sidebars
        let open_sidebars   = open_sidebars.clone();
        let panel_changes   = panels.follow_changes();

        // Pipe them into the runner
        pipe_in(Arc::clone(&runner), panel_changes, move |_, changes| {
            let open_sidebars = open_sidebars.clone();
            async move {
                use self::RopeAction::*;

                match changes {
                    SetAttributes(_, _)         => { }
                    ReplaceAttributes(_, _, _)  => { }
                    Replace(_, cells)           => {
                        // Get the open panels
                        let mut open_panels     = open_sidebars.get();
                        let mut open_changed    = false;

                        for panel in cells.into_iter().rev() {
                            if panel.active().get() {
                                if !open_panels.contains(&panel.identifier().to_string()) {
                                    open_panels.insert(0, panel.identifier().to_string());
                                    open_changed = true;
                                }
                            }
                        }

                        // Update the open panels if they've changed
                        if open_changed {
                            open_sidebars.set(open_panels);
                        }
                    }
                }
            }.boxed()
        });

        runner
    }

    ///
    /// Returns a binding representing the activation state of the sidebar panel as a whole
    ///
    fn activation_state(panels: &RopeBinding<SidebarPanel, ()>) -> BindRef<SidebarActivationState> {
        // Map to a list of 'is active' bindings
        let panels              = panels.clone();
        let activation_state    = computed(move || {
            let panels          = panels.get();
            let is_active       = panels.read_cells(0..panels.len())
                .any(|panel| panel.active().get());

            if is_active {
                SidebarActivationState::Active
            } else {
                SidebarActivationState::Inactive
            }
        });

        BindRef::from(activation_state)
    }
}

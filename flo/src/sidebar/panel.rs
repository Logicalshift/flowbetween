use flo_ui::*;
use flo_binding::*;

use std::sync::*;

///
/// Describes a panel that can be displayed on the sidebar
///
#[derive(Clone)]
pub struct SidebarPanel {
    /// Unique identifier for this panel
    identifier: String,

    /// Title for this panel as it appears in the 
    title: String,

    /// The icon for this sidebar panel (or None to just use the title)
    icon: Option<Image>,

    /// The height in pixels of the panel (width is always ~300)
    height: BindRef<f64>,

    /// Binding indicating whether or not this panel is 'active' (has settings relevant to the current context)
    active: BindRef<bool>,

    /// The controller for the content of this panel
    controller: Arc<dyn Controller>,
}

impl PartialEq for SidebarPanel {
    fn eq(&self, panel: &SidebarPanel) -> bool {
        // Panels are the same if they have the same identifier
        self.identifier.eq(&panel.identifier)
    }
}

impl SidebarPanel {
    ///
    /// Returns the controller for this panel
    ///
    pub fn controller(&self) -> Arc<dyn Controller> {
        Arc::clone(&self.controller)
    }

    ///
    /// The unique identifier for this panel
    ///
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    ///
    /// The title for this panel
    ///
    pub fn title(&self) -> &str {
        &self.title
    }

    ///
    /// The binding containing the height of this panel
    ///
    pub fn height(&self) -> &BindRef<f64> {
        &self.height
    }

    ///
    /// The image for the icon for this panel, if it has one
    ///
    pub fn icon(&self) -> &Option<Image> {
        &self.icon
    }

    ///
    /// The binding that indicates whether or not this panel is active
    ///
    pub fn active(&self) -> &BindRef<bool> {
        &self.active
    }
}
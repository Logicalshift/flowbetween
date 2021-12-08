use flo_ui::*;
use flo_binding::*;

use uuid::*;

use std::sync::*;
use std::cmp::{Ordering};
use std::hash::{Hash, Hasher};

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

impl Eq for SidebarPanel {

}

impl PartialOrd for SidebarPanel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.identifier.partial_cmp(&other.identifier)
    }
}

impl Ord for SidebarPanel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identifier.cmp(&other.identifier)
    }
}

impl Hash for SidebarPanel {
    fn hash<H>(&self, state: &mut H)
    where H: Hasher {
        self.identifier.hash(state)
    }
}

impl SidebarPanel {
    ///
    /// Creates a sidebar panel with a particular name
    ///
    pub fn with_title(title: &str) -> Self {
        let identifier = Uuid::new_v4().to_simple().to_string();

        SidebarPanel {
            identifier: identifier,
            title:      title.into(),
            icon:       None,
            height:     BindRef::from(bind(96.0)),
            active:     BindRef::from(bind(false)),
            controller: Arc::new(EmptyController)
        }
    }

    ///
    /// Sets the controller for a sidebar panel (this describes the content of the control when it's open in the sidebar)
    ///
    pub fn with_controller<TController: 'static+Controller>(mut self, controller: TController) -> Self {
        self.controller = Arc::new(controller);
        self
    }

    ///
    /// Adds an image to a sidebar panel
    ///
    pub fn with_icon(mut self, icon: Image) -> Self {
        self.icon = Some(icon);
        self
    }

    ///
    /// Modifies a sidebar panel with a height binding
    ///
    pub fn with_height(mut self, height: BindRef<f64>) -> Self {
        self.height = height;
        self
    }

    ///
    /// Modifies a sidebar panel with an active binding
    ///
    pub fn with_active(mut self, active: BindRef<bool>) -> Self {
        self.active = active;
        self
    }

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
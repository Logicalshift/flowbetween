use flo_ui::*;
use flo_binding::*;

///
/// Describes a panel that can be displayed on the sidebar
///
pub struct SidebarPanel {
    /// Unique identifier for this panel
    identifier: String,

    /// Title for this panel as it appears in the 
    title: String,

    /// The icon for this sidebar panel (or None to just use the title)
    icon: Option<Image>,

    /// Binding indicating whether or not this panel is 'active' (has settings relevant to the current context)
    active: BindRef<bool>,
}

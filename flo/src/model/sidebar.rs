use flo_binding::*;

///
/// Model representing the state of the sidebar controller
///
#[derive(Clone)]
pub struct SidebarModel {
    /// Whether or not the sidebar has been opened by the user
    pub is_open: Binding<bool>,

    /// List of identifiers in priority order of the sidebar items that are open (hidden sidebars can be specified as open, sidebars are collapsed in priority order when they won't all fit on screen)
    pub open_sidebars: Binding<Vec<String>>
}

impl SidebarModel {
    ///
    /// 
    ///
    pub fn new() -> SidebarModel {
        SidebarModel {
            is_open:        bind(false),
            open_sidebars:  bind(vec![])
        }
    }
}

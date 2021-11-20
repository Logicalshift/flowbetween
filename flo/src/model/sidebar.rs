use flo_binding::*;

///
/// Model representing the state of the sidebar controller
///
#[derive(Clone)]
pub struct SidebarModel {
    /// Whether or not the sidebar has been opened by the user
    is_open: Binding<bool>
}

impl SidebarModel {
    ///
    /// 
    ///
    pub fn new() -> SidebarModel {
        SidebarModel {
            is_open: bind(false)
        }
    }
}
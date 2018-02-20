use super::tool_action::*;
use super::tool_input::*;

///
/// Trait implemented by something representing a tool
/// 
pub trait Tool2 {
    ///
    /// Retrieves the name of this tool
    /// 
    fn tool_name(&self) -> String;

    ///
    /// Retrieves the name of the image that is associated with this tool
    /// 
    fn image_name(&self) -> String;

    ///
    /// Retrieves the menu controller to use for adjusting this tool
    /// 
    fn menu_controller_name(&self) -> String { "".to_string() }
}

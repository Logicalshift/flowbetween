use ui::*;
use animation::*;

use std::sync::*;

mod select;
mod adjust;
mod pan;
mod pencil;
mod ink;

use self::select::*;
use self::adjust::*;
use self::pan::*;
use self::pencil::*;
use self::ink::*;

///
/// Trait implemented by tool objects
/// 
pub trait Tool : Send+Sync {
    ///
    /// Retrieves the name of this tool
    /// 
    fn tool_name(&self) -> String;

    ///
    /// Retrieves the name of the image that is associated with this tool
    /// 
    fn image_name(&self) -> String;

    ///
    /// User is painting with this tool selected alongside a particular layer
    /// 
    fn paint(&self, selected_layer: Arc<Layer>, device: &PaintDevice, actions: &Vec<Painting>);
}

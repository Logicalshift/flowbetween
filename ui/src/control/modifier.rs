use super::control::*;
use super::attributes::*;

///
/// Represents an object that can be used to modify a control
/// 
pub trait ControlModifier {
    /// Applies this modifier to a control
    fn modify(self, control: &Control);
}

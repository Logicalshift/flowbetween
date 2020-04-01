use super::edit::*;
use super::motion::*;

///
/// Supplies the motion elements for an animation
///
pub trait AnimationMotion {
    ///
    /// Retrieves the IDs of the motions attached to a particular element
    ///
    fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId>;

    ///
    /// Retrieves the IDs of the elements attached to a particular motion
    ///
    fn get_elements_for_motion(&self, motion_id: ElementId) -> Vec<ElementId>;

    ///
    /// Retrieves the motion with the specified ID
    ///
    fn get_motion(&self, motion_id: ElementId) -> Option<Motion>;
}

///
/// Represents an edit to an element within a frame
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum ElementEdit {
    /// Updates the control points for this element
    SetControlPoints(Vec<(f32, f32)>)
}

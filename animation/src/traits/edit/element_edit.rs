///
/// Represents an edit to an element within a frame
/// 
#[derive(Clone, PartialEq, Debug)]
pub enum ElementEdit {
    /// Element should be moved such that it fills a particular rectangle
    Move((f32, f32), (f32, f32))
}

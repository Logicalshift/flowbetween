///
/// A particular mouse button
/// 
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u32)
}

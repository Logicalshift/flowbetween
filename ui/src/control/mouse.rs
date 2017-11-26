///
/// A particular mouse button
/// 
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u32)
}

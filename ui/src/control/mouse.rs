///
/// A particular mouse button
///
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Other(u32)
}

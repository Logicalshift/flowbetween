///
/// Modifier keys supported by the UI
///
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ModifierKey {
    Shift,
    Ctrl,
    Alt,
    Meta,
    Super,
    Hyper
}

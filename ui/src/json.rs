use serde_json;

///
/// Trait implemented by types that can be converted to and from a JSON value representation
///
pub trait ToJsonValue {
    ///
    /// Creates a JSON representation of this item
    ///
    fn to_json(&self) -> serde_json::Value;

    ///
    /// Reconstitutes this value from its JSON representation
    ///
    fn from_json(json: &serde_json::Value);
}

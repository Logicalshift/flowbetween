mod ink;

pub use self::ink::*;

///
/// Stores the definition of a particular brush
/// 
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum BrushDefinition {
    /// Represents the definition of an ink brush
    Ink(InkDefinition)
}

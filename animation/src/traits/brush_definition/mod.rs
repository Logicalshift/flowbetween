mod ink;

pub use self::ink::*;

///
/// Stores the definition of a particular brush
///
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum BrushDefinition {
    /// The simple brush is usually only used for testing
    Simple,

    /// An ink brush with a particular definition
    Ink(InkDefinition)
}

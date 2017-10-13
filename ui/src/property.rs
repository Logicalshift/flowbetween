use serde::*;

///
/// Represents a property (a control value that can either be a
/// constant or a viewmodel value)
///
#[derive(Clone)]
pub enum Property {
    Int(i32),
    Float(f64),
    String(String),

    /// Property is bound to a value in the view model
    Bind(String)
}

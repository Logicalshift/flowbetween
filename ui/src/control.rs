///
/// Represents an element in the user interface
///
pub enum Control {
}

///
/// Possible positions of a control
///
pub enum ControlPosition {
    /// Control located at a particular point
    At(f32, f32),

    /// Control located at the start of its container
    Start,

    /// Control located at the end of its container
    End,

    /// Control positioned immediately after the last control
    After
}

use animation::*;

///
/// Represents an editing action for a tool
/// 
pub enum ToolAction<ToolData> {
    /// Changes the data that will be specified at the start of the next tool input stream
    Data(ToolData),

    /// Specifies a series of edits to perform
    Edit(AnimationEdit)
}

use ui::*;

///
/// Represents an input to a tool
///
pub enum ToolInput<'a, ToolData: 'a> {
    /// Specifies the data set for this tool
    Data(&'a ToolData),

    /// Specifies painting on a specific device
    PaintDevice(PaintDevice),

    /// Specifies an input paint action
    Paint(PaintAction)
}

use ui::*;

use std::sync::*;

///
/// Represents an input to a tool
///
#[derive(Debug)]
pub enum ToolInput<ToolData> {
    /// Specifies the data set for this tool
    Data(Arc<ToolData>),

    /// Specifies painting on a specific device
    PaintDevice(PaintDevice),

    /// Specifies an input paint action
    Paint(Painting)
}

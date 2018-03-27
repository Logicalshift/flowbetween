use super::super::gtk_action::*;

///
/// Represents a control within the
/// 
pub struct GtkControl {
    /// The ID of the widget that is displaying the UI for this control
    pub widget_id: WidgetId,

    /// The name of the controller that this control and its children belong to
    pub controller: Option<String>,

    /// Controls stored underneath this one in the UI tree
    pub child_controls: Vec<GtkControl>
}

impl GtkControl {
    ///
    /// Creates a new GTK control with a particular widget ID
    /// 
    pub fn new(widget_id: WidgetId) -> GtkControl {
        GtkControl {
            widget_id:      widget_id,
            controller:     None,
            child_controls: vec![]
        }
    }
}

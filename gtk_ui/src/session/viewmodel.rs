use super::super::gtk_action::*;

use futures::*;

///
/// Tracks and generates events for viewmodel changes in GTK
/// 
pub struct GtkSessionViewModel {
    /// The sink where the actions for this viewmodel should go
    action_sink: Box<Sink<SinkItem=Vec<GtkAction>, SinkError=()>>
}

impl GtkSessionViewModel {
    ///
    /// Creates a new GTK sesion viewmodel, which will send events to the specified sink
    /// 
    pub fn new(action_sink: Box<Sink<SinkItem=Vec<GtkAction>, SinkError=()>> ) -> GtkSessionViewModel {
        GtkSessionViewModel {
            action_sink: action_sink
        }
    }
}
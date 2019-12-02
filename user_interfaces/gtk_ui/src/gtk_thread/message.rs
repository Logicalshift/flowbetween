use super::flo_gtk::*;

///
/// Trait implemented by objects that can process FloGtkMessages
///
pub trait FloGtkMessage : Send {
    /// Processes this message
    fn process(&mut self, flo_gtk: &mut FloGtk);
}

///
/// Represents a FloGtkMessage that is processed by evaluating a FnOnce
///
pub struct FnOnceMessage<TAction> {
    process: Option<TAction>
}

impl<TAction> FnOnceMessage<TAction> {
    ///
    /// Creates a new FnOnceMessage
    ///
    pub fn new(action: TAction) -> FnOnceMessage<TAction> {
        FnOnceMessage {
            process: Some(action)
        }
    }
}

impl<TAction> FloGtkMessage for FnOnceMessage<TAction>
where TAction: 'static+Send+FnOnce(&mut FloGtk) -> () {
    fn process(&mut self, flo_gtk: &mut FloGtk) {
        if let Some(process) = self.process.take() {
            process(flo_gtk);
        }
    }
}

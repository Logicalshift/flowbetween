use super::traits::*;

use std::sync::*;
use std::cell::*;

struct NotifyFn<TFn> {
    when_changed: Mutex<RefCell<TFn>>
}

impl<TFn> Notifiable for NotifyFn<TFn>
where TFn: Send+FnMut() -> () {
    fn mark_as_changed(&self) {
        let cell        = self.when_changed.lock().unwrap();
        let on_changed  = &mut *cell.borrow_mut();
        
        on_changed()
    }
}

///
/// Creates a notifiable reference from a function
///
pub fn notify<TFn>(when_changed: TFn) -> Arc<Notifiable>
where TFn: 'static+Send+FnMut() -> () {
    Arc::new(NotifyFn { when_changed: Mutex::new(RefCell::new(when_changed)) })
}

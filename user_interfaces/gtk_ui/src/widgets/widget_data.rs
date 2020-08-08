use super::widget::*;
use super::super::gtk_action::*;

use anymap::*;

use std::rc::*;
use std::cell::*;
use std::collections::HashMap;
use std::ops::Deref;

///
/// Represents a data entry for a widget
///
pub struct WidgetDataEntry<TData> {
    /// Reference to the data in this entry
    data: Rc<RefCell<TData>>,
}

impl<TData> Deref for WidgetDataEntry<TData> {
    type Target = Rc<RefCell<TData>>;

    fn deref(&self) -> &Rc<RefCell<TData>> {
        &self.data
    }
}

///
/// Used to associate Flo data with widgets
///
/// (An annoying thing with Gtk is that in order to track things like the Flo layout and property
/// bindings for a widget, we need to either subclass them all or track their data independently.
/// Here, we track their data manually)
///
pub struct WidgetData {
    // We store our values in RefCells so that we can retrieve and add values via a 'self' reference
    // rather than a 'mut self' reference (making this convenient to use from a reference that's
    // passed around GTK signal handlers, but otherwise a bit ugly)

    /// Hashmap for the widgets that are being managed by this object
    widgets: RefCell<HashMap<WidgetId, Rc<RefCell<dyn GtkUiWidget>>>>,

    /// Data attached to a particular widget ID
    widget_data: RefCell<HashMap<WidgetId, AnyMap>>
}

impl WidgetData {
    ///
    /// Creates a new widget data object
    ///
    pub fn new() -> WidgetData {
        WidgetData {
            widgets:        RefCell::new(HashMap::new()),
            widget_data:    RefCell::new(HashMap::new())
        }
    }

    ///
    /// Associates a widget with an ID
    ///
    pub fn register_widget<TWidget: 'static+GtkUiWidget>(&self, widget_id: WidgetId, widget: TWidget) {
        self.widgets.borrow_mut().insert(widget_id, Rc::new(RefCell::new(widget)));
        self.widget_data.borrow_mut()
            .entry(widget_id)
            .or_insert_with(|| AnyMap::new());
    }

    ///
    /// Substitutes a different widget for the specified widget
    ///
    pub fn replace_widget<TWidget: 'static+GtkUiWidget>(&self, widget_id: WidgetId, widget: TWidget) {
        self.widgets.borrow_mut()
            .get_mut(&widget_id)
            .map(move |widget_ref| *widget_ref = Rc::new(RefCell::new(widget)));
    }

    ///
    /// Attempts to retrieve the widget with the specified ID
    ///
    pub fn get_widget(&self, widget_id: WidgetId) -> Option<Rc<RefCell<dyn GtkUiWidget>>> {
        self.widgets.borrow().get(&widget_id).cloned()
    }

    ///
    /// Removes the widget that has the specified ID
    ///
    pub fn remove_widget(&self, widget_id: WidgetId) {
        self.widgets.borrow_mut().remove(&widget_id);
        self.widget_data.borrow_mut().remove(&widget_id);
    }

    ///
    /// Sets the data associated with a particular type and widget
    ///
    pub fn set_widget_data<TData: 'static>(&self, widget_id: WidgetId, new_data: TData) {
        self.widget_data.borrow_mut()
            .entry(widget_id)
            .or_insert_with(|| AnyMap::new())
            .insert(Rc::new(RefCell::new(new_data)));
    }

    ///
    /// Retrieves the data of a specific type associated with a widget
    ///
    pub fn get_widget_data<'a, TData: 'static>(&'a self, widget_id: WidgetId) -> Option<WidgetDataEntry<TData>> {
        self.widget_data.borrow_mut()
            .get_mut(&widget_id)
            .and_then(move |anymap| anymap.get::<Rc<RefCell<TData>>>())
            .map(|data| WidgetDataEntry { data: Rc::clone(&data) })
    }

    ///
    /// Retrieves the data of a specific type associated with a widget
    ///
    pub fn get_widget_data_or_insert<'a, TData: 'static, FnInsert: FnOnce() -> TData>(&'a self, widget_id: WidgetId, or_insert: FnInsert) -> Option<WidgetDataEntry<TData>> {
        self.widget_data.borrow_mut()
            .get_mut(&widget_id)
            .map(move |anymap| anymap.entry::<Rc<RefCell<TData>>>().or_insert_with(move || Rc::new(RefCell::new(or_insert()))))
            .map(|data| WidgetDataEntry { data: Rc::clone(&data) })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::gtk_thread::*;
    use gtk;

    struct TestWidget { }

    impl GtkUiWidget for TestWidget {
        fn id(&self) -> WidgetId { WidgetId::Assigned(0) }
        fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) { }
        fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) { }
        fn get_underlying<'a>(&'a self) -> &'a gtk::Widget { unimplemented!() }
    }

    #[test]
    fn non_existent_widget() {
        let data = WidgetData::new();
        assert!(data.get_widget(WidgetId::Assigned(42)).is_none());
    }

    #[test]
    fn create_widget() {
        let data = WidgetData::new();
        data.register_widget(WidgetId::Assigned(42), TestWidget {});
        assert!(data.get_widget(WidgetId::Assigned(42)).is_some());
    }

    #[test]
    fn store_data_for_widget() {
        let data = WidgetData::new();
        data.register_widget(WidgetId::Assigned(42), TestWidget {});
        data.set_widget_data(WidgetId::Assigned(42), 42);

        let value = data.get_widget_data::<i32>(WidgetId::Assigned(42)).unwrap();
        assert!(*value.borrow() == 42);
    }

    #[test]
    fn different_data_types() {
        let data = WidgetData::new();
        data.register_widget(WidgetId::Assigned(42), TestWidget {});
        data.set_widget_data(WidgetId::Assigned(42), 42);
        data.set_widget_data(WidgetId::Assigned(42), String::from("Hello, world"));

        let value = data.get_widget_data::<i32>(WidgetId::Assigned(42)).unwrap();
        assert!(*value.borrow() == 42);

        let value = data.get_widget_data::<String>(WidgetId::Assigned(42)).unwrap();
        assert!(&*value.borrow() == &String::from("Hello, world"));
    }
}

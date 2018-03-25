use super::widget::*;
use super::super::gtk_action::*;

use anymap::*;

use std::rc::*;
use std::cell::*;
use std::collections::HashMap;

///
/// Represents a mutable borrow of the data associated with a widget
/// 
pub struct WidgetDataRef<'a, TData: 'a> {
    widget_ref: RefMut<'a, HashMap<WidgetId, AnyMap>>,
    data:       &'a mut TData
}

///
/// Used to associate Flo data with widgets
/// 
/// (An annoying thing with Gtk is that in order to track things like the Flo layout and property
/// bindings for a widget, we need to either subclass them all or track their data independently.
/// Here, we track their data manually)
/// 
pub struct WidgetData {
    /// Hashmap for the widgets that are being managed by this object
    widgets: RefCell<HashMap<WidgetId, Rc<RefCell<GtkUiWidget>>>>,

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
        self.widget_data.borrow_mut().insert(widget_id, AnyMap::new());
    }

    ///
    /// Attempts to retrieve the widget with the specified ID
    /// 
    pub fn get_widget(&self, widget_id: WidgetId) -> Option<Rc<RefCell<GtkUiWidget>>> {
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
            .get_mut(&widget_id)
            .map(move |anymap| anymap.insert(new_data));
    }

    ///
    /// Retrieves the data of a specific type associated with a widget
    /// 
    pub fn get_widget_data<'a, TData: 'static>(&'a self, widget_id: WidgetId) -> Option<&'a mut TData> {
        self.widget_data.borrow_mut()
            .get_mut(&widget_id)
            .and_then(move |anymap| anymap.get_mut::<TData>())
    }

    ///
    /// Retrieves the data of a specific type associated with a widget
    /// 
    pub fn get_widget_data_or_insert<'a, TData: 'static, FnInsert: FnOnce() -> TData>(&'a self, widget_id: WidgetId, or_insert: FnInsert) -> Option<&'a mut TData> {
        self.widget_data.borrow_mut()
            .get_mut(&widget_id)
            .map(move |anymap| anymap.entry::<TData>().or_insert_with(or_insert))
    }
}

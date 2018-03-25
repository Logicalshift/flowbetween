use super::layout::*;
use super::widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Represents a basic widget
/// 
pub struct BasicWidget(pub WidgetId, pub gtk::Widget);

impl BasicWidget {
    ///
    /// Creates a basic widget
    /// 
    pub fn new<Src: Cast+IsA<gtk::Widget>>(id: WidgetId, widget: Src) -> BasicWidget {
        BasicWidget(id, widget.upcast::<gtk::Widget>())
    }
}

impl GtkUiWidget for BasicWidget {
    fn id(&self) -> WidgetId {
        let BasicWidget(id, ref _widget) = *self;
        id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        let BasicWidget(id, ref mut widget) = *self;

        process_basic_widget_action(id, widget, flo_gtk, action);
    }

    fn add_child(&mut self, new_child: Rc<RefCell<GtkUiWidget>>) {
        let BasicWidget(_id, ref widget) = *self;

        // Remove the child widget from its existing parent
        let new_child = new_child.borrow();
        let new_child = new_child.get_underlying();

        new_child.unparent();

        // If this widget is a container, add this as a child widget
        let container = widget.clone().dynamic_cast::<gtk::Container>();
        if let Ok(container) = container {
            container.add(new_child);
        }
    }

    fn set_parent(&mut self, _new_parent: Rc<RefCell<GtkUiWidget>>) {
        // Basic widgets don't need to know what their parent is
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        let BasicWidget(_id, ref widget) = *self;
        widget
    }
}

///
/// Performs the basic processing associated with a widget action (using a generic Gtk widget as the target)
/// 
pub fn process_basic_widget_action(id: WidgetId, widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
    use self::GtkWidgetAction::*;

    match action {
        &Layout(ref layout)         => process_basic_widget_layout(id, widget, flo_gtk, layout),
        &Content(ref content)       => process_basic_widget_content(id, widget, flo_gtk, content),
        &Appearance(ref appearance) => process_basic_widget_appearance(widget, flo_gtk, appearance),
        &State(ref state)           => process_basic_widget_state(widget, flo_gtk, state),
        &Font(ref font)             => process_basic_widget_font(widget, flo_gtk, font),

        &New(_widget_type)          => (),
        &SetRoot(window_id)         => { flo_gtk.get_window(window_id).map(|window| window.borrow_mut().set_root(flo_gtk, widget)); },
        &Delete                     => { widget.unparent(); }
    }
}

///
/// Processes a layout command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_layout(id: WidgetId, widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, layout: &WidgetLayout) { 
    // Fetch or create the layout for this widget
    let widget_data     = flo_gtk.widget_data();
    let widget_layout   = widget_data.get_widget_data_or_insert(id, || Layout::new());

    // Update it with the content of the command
    widget_layout.map(move |widget_layout| widget_layout.borrow_mut().update(layout));

    // Tell the parent of this widget it needs relayout
    widget.queue_resize();
}

///
/// Performs the actions required to set a widget's parent
/// 
pub fn set_widget_parent(widget_id: WidgetId, new_parent_id: WidgetId, flo_gtk: &mut FloGtk) {
    // Fetch the widget information
    let widget_data     = flo_gtk.widget_data();
    let child_widget    = widget_data.get_widget(widget_id);
    let parent_widget   = widget_data.get_widget(new_parent_id);

    if let (Some(child_widget), Some(parent_widget)) = (child_widget, parent_widget) {
        // Set the parent of the child widget
        child_widget.borrow_mut().set_parent(Rc::clone(&parent_widget));

        // Add to the children of the new parent widget
        parent_widget.borrow_mut().add_child(Rc::clone(&child_widget));
    }
}

///
/// Processes a content command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_content(id: WidgetId, widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, content: &WidgetContent) {
    use self::WidgetContent::*;

    match content {
        &SetParent(parent_id)   => set_widget_parent(id, parent_id, flo_gtk),
        &SetText(ref _text)     => () /* Standard gtk widgets can't have text in them */,
        &Draw(ref canvas)       => unimplemented!()
    }
}

///
/// Generic appearance command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_appearance(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, appearance: &Appearance) {
    use self::Appearance::*;

    match appearance {
        &Foreground(ref color)      => (),
        &Background(ref color)      => (),
        &Image(ref color)           => ()
    }
}

///
/// Processes a basic state command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_state(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, state: &State) {
    use self::State::*;

    match state {
        &Selected(ref selected_prop)    => (),
        &Badged(ref badged_prop)        => (),
        &Value(ref value_prop)          => (),
        &Range((ref from, ref to))      => ()
    }
}

///
/// Processes a font command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_font(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, font: &Font) {
    use self::Font::*;

    match font {
        &Size(size_pixels)      => (),
        &Align(ref align)       => (),
        &Weight(ref weight)     => ()
    }
}

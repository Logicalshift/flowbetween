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
        process_basic_widget_action(self, flo_gtk, action);
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        let BasicWidget(_id, ref widget) = *self;

        // If this widget is a container, add this as a child widget
        let container = widget.clone().dynamic_cast::<gtk::Container>();
        if let Ok(container) = container {
            // Remove any existing child widgets
            container.get_children().iter().for_each(|child| container.remove(child));

            for new_child in children {
                // Remove the child widget from its existing parent
                let new_child = new_child.borrow();
                let new_child = new_child.get_underlying();

                new_child.unparent();

                // Add to the container
                container.add(new_child);
            }
        }
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        let BasicWidget(_id, ref widget) = *self;
        widget
    }
}

///
/// Performs the basic processing associated with a widget action (using a generic Gtk widget as the target)
/// 
pub fn process_basic_widget_action<W: GtkUiWidget>(widget: &mut W, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
    use self::GtkWidgetAction::*;

    match action {
        &Layout(ref layout)         => process_basic_widget_layout(widget.id(), widget.get_underlying(), flo_gtk, layout),
        &Content(ref content)       => process_basic_widget_content(widget, flo_gtk, content),
        &Appearance(ref appearance) => process_basic_widget_appearance(widget.get_underlying(), flo_gtk, appearance),
        &State(ref state)           => process_basic_widget_state(widget.get_underlying(), flo_gtk, state),
        &Font(ref font)             => process_basic_widget_font(widget.get_underlying(), flo_gtk, font),
        &Scroll(ref scroll)         => process_basic_widget_scroll(widget.get_underlying(), flo_gtk, scroll),

        &New(_widget_type)          => (),
        &SetRoot(window_id)         => { 
            let widget = widget.get_underlying().clone();
            flo_gtk.get_window(window_id).map(|window| window.borrow_mut().set_root(flo_gtk, &widget));
        },
        &Delete                     => { widget.get_underlying().unparent(); }
    }
}

///
/// Processes a layout command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_layout<W: WidgetExt>(id: WidgetId, widget: &W, flo_gtk: &mut FloGtk, layout: &WidgetLayout) { 
    // Fetch or create the layout for this widget
    let widget_data     = flo_gtk.widget_data();
    let widget_layout   = widget_data.get_widget_data_or_insert(id, || Layout::new());

    // Update it with the content of the command
    widget_layout.map(move |widget_layout| widget_layout.borrow_mut().update(layout));

    // Tell the parent of this widget it needs relayout
    widget.get_parent().map(|parent| parent.queue_resize());
}

///
/// Performs the actions required to set a widget's parent
/// 
pub fn set_widget_parent<W: GtkUiWidget>(widget: &mut W, children: &Vec<WidgetId>, flo_gtk: &mut FloGtk) {
    // Fetch the widget information
    let widget_data     = flo_gtk.widget_data();
    let children        = children.iter()
        .map(|child_id| widget_data.get_widget(*child_id))
        .filter(|child| !child.is_none())
        .map(|child| child.unwrap())
        .collect();
    
    widget.set_children(children);
}

///
/// Processes a content command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_content<W: GtkUiWidget>(widget: &mut W, flo_gtk: &mut FloGtk, content: &WidgetContent) {
    use self::WidgetContent::*;

    match content {
        &SetChildren(ref children)      => set_widget_parent(widget, children, flo_gtk),
        &SetText(ref _text)             => () /* Standard gtk widgets can't have text in them */,
        &AddClass(ref class_name)       => unimplemented!(),
        &RemoveClass(ref class_name)    => unimplemented!(),
        &Draw(ref canvas)               => unimplemented!()
    }
}

///
/// Generic appearance command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_appearance<W: WidgetExt>(widget: &W, flo_gtk: &mut FloGtk, appearance: &Appearance) {
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
pub fn process_basic_widget_state<W: WidgetExt>(widget: &W, flo_gtk: &mut FloGtk, state: &WidgetState) {
    use self::WidgetState::*;

    match state {
        &SetSelected(selected)      => (),
        &SetBadged(badged)          => (),
        &SetValueFloat(value)       => (),
        &SetRangeMin(from)          => (),
        &SetRangeMax(to)            => ()
    }
}

///
/// Processes a font command for a widget being managed by FlowBetween
/// 
pub fn process_basic_widget_font<W: WidgetExt>(widget: &W, flo_gtk: &mut FloGtk, font: &Font) {
    use self::Font::*;

    match font {
        &Size(size_pixels)      => (),
        &Align(ref align)       => (),
        &Weight(ref weight)     => ()
    }
}

pub fn process_basic_widget_scroll<W: WidgetExt>(widget: &W, flo_gtk: &mut FloGtk, scroll: &Scroll) {
    use self::Scroll::*;

    match scroll {
        &MinimumContentSize(width, height)      => (),
        &HorizontalScrollBar(ref visibility)    => (),
        &VerticalScrollBar(ref visibility)      => (),
        &Fix(ref axis)                          => ()
    }
}

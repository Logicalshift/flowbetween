use super::drag::*;
use super::click::*;
use super::paint::*;
use super::layout::*;
use super::widget::*;
use super::flo_layout::*;
use super::custom_style::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;
use super::super::gtk_widget_event_type::*;

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

    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
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

                let previous_parent = new_child.get_parent().and_then(|parent| parent.dynamic_cast::<gtk::Container>().ok());
                previous_parent.map(|previous_parent| previous_parent.remove(new_child));

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
        &RequestEvent(event_type, ref action_name)  => process_basic_event_request(widget, flo_gtk, event_type, action_name),
        &Layout(ref layout)                         => process_basic_widget_layout(widget.id(), widget.get_underlying(), flo_gtk, layout),
        &Content(ref content)                       => process_basic_widget_content(widget, flo_gtk, content),
        &Appearance(ref appearance)                 => process_basic_widget_appearance(widget, flo_gtk, appearance),
        &State(ref state)                           => process_basic_widget_state(widget, state),
        &Font(ref font)                             => process_basic_widget_font(widget, flo_gtk, font),
        &Scroll(ref scroll)                         => process_basic_widget_scroll(widget.get_underlying(), flo_gtk, scroll),
        &Popup(ref _popup)                          => (),

        &Show                                       => { widget.get_underlying().show() },
        &New(_widget_type)                          => (),
        &Delete                                     => {
            // Remove this widget from its parent
            let widget          = widget.get_underlying();
            let previous_parent = widget.get_parent().and_then(|parent| parent.dynamic_cast::<gtk::Container>().ok());
            previous_parent.map(|previous_parent| previous_parent.remove(widget));
        },

        &SetRoot(window_id)                         => {
            let widget = widget.get_underlying().clone();
            flo_gtk.get_window(window_id).map(|window| window.borrow_mut().set_root(flo_gtk, &widget));
        },

        &IntoEventBox                               => { }
    }
}

///
/// Processes a layout command for a widget being managed by FlowBetween
///
pub fn process_basic_widget_layout<W: Clone+WidgetExt+IsA<gtk::Widget>>(id: WidgetId, widget: &W, flo_gtk: &mut FloGtk, layout: &WidgetLayout) {
    // Fetch or create the layout for this widget
    let widget_data     = flo_gtk.widget_data();
    let widget_layout   = widget_data.get_widget_data_or_insert(id, || Layout::new());

    // Update it with the content of the command
    widget_layout.map(move |widget_layout| widget_layout.borrow_mut().update(layout));

    // For floating widgets, we may need to reallocate them immediately
    if let &WidgetLayout::Floating(float_x, float_y) = layout {
        // Update the floating position data (so the next layout will use it)
        widget_data.set_widget_data(id, FloatingPosition { x: float_x, y: float_y });

        // Update the widget position from its position
        if let Some(current_position) = widget_data.get_widget_data::<WidgetPosition>(id) {
            let current_position    = current_position.borrow();

            let new_x               = current_position.x1 + float_x;
            let new_y               = current_position.y1 + float_y;
            let width               = current_position.x2 - current_position.x1;
            let height              = current_position.y2 - current_position.y2;

            let new_x               = new_x.floor() as i32;
            let new_y               = new_y.floor() as i32;
            let width               = width.floor() as i32;
            let height              = height.floor() as i32;

            // Size the widget
            let widget              = widget.clone().upcast::<gtk::Widget>();
            let parent              = widget.get_parent();
            let event_box           = parent.as_ref().and_then(|parent| parent.clone().dynamic_cast::<gtk::EventBox>().ok());
            let (parent, widget)    = if let Some(event_box) = event_box { (event_box.get_parent(), event_box.upcast()) } else { (parent, widget) };

            let fixed       = parent.as_ref().and_then(|parent| parent.clone().dynamic_cast::<gtk::Fixed>().ok());
            let layout      = parent.and_then(|parent| parent.dynamic_cast::<gtk::Layout>().ok());

            widget.size_allocate(&mut gtk::Rectangle { x: new_x, y: new_y, width: width, height: height });

            if let Some(fixed) = fixed {
                fixed.move_(&widget, new_x, new_y);
            }
            if let Some(layout) = layout {
                layout.move_(&widget, new_x, new_y);
            }
        }
    }

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
        &Draw(ref _drawing)             => () /* Drawing requires support from the widget */,

        &AddClass(ref class_name)       => {
            let widget          = widget.get_underlying();
            let style_context   = widget.get_style_context();
            style_context.add_class(&*class_name);
        },

        &RemoveClass(ref class_name)    => {
            let widget          = widget.get_underlying();
            let style_context   = widget.get_style_context();
            style_context.remove_class(&*class_name);
        }
    }
}

///
/// Generic appearance command for a widget being managed by FlowBetween
///
pub fn process_basic_widget_appearance<W: GtkUiWidget>(widget: &W, flo_gtk: &mut FloGtk, appearance: &Appearance) {
    use self::Appearance::*;

    match appearance {
        &Foreground(ref color)      => {
            let custom_style = flo_gtk.widget_data().get_custom_style(widget);
            custom_style.borrow_mut().set_foreground(color);
        },

        &Background(ref color)      => {
            let custom_style = flo_gtk.widget_data().get_custom_style(widget);
            custom_style.borrow_mut().set_background(color);
        },

        &Image(ref _image)          => ()
    }
}

///
/// Processes a basic state command for a widget being managed by FlowBetween
///
pub fn process_basic_widget_state<W: GtkUiWidget>(widget: &W, state: &WidgetState) {
    use self::WidgetState::*;

    match state {
        &SetSelected(selected)      => {
            let context = widget.get_underlying()
                .get_style_context();
            if selected { context.add_class("selected") } else { context.remove_class("selected") }

            // TODO: toggle buttons probably should get their own class thing
            widget.get_underlying().clone().dynamic_cast::<gtk::ToggleButton>().ok().map(|toggle| { toggle.set_active(selected); });
        },
        &SetBadged(badged)          => {
            let context = widget.get_underlying()
                .get_style_context();
            if badged { context.add_class("badged") } else { context.remove_class("badged") }
        },
        &SetEnabled(enabled)        => {
            widget.get_underlying()
                .set_sensitive(enabled)
        },

        SetValueBool(_value)        => (),
        SetValueInt(_value)         => (),
        SetValueText(_value)        => (),
        SetValueFloat(_value)       => (),
        SetRangeMin(_from)          => (),
        SetRangeMax(_to)            => ()
    }
}

///
/// Processes a font command for a widget being managed by FlowBetween
///
pub fn process_basic_widget_font<W: GtkUiWidget>(widget: &W, flo_gtk: &mut FloGtk, font: &Font) {
    use self::Font::*;

    match font {
        &Align(_align)          => (),
        &Size(size_pixels)      => {
            let custom_style = flo_gtk.widget_data().get_custom_style(widget);
            custom_style.borrow_mut().set_font_size(size_pixels);
        },
        &Weight(weight)         =>  {
            let custom_style = flo_gtk.widget_data().get_custom_style(widget);
            custom_style.borrow_mut().set_font_weight(weight as u32);
        }
    }
}

///
/// Processes a scroll command for a widget
///
pub fn process_basic_widget_scroll<W: WidgetExt>(_widget: &W, _flo_gtk: &mut FloGtk, scroll: &Scroll) {
    use self::Scroll::*;

    match scroll {
        &MinimumContentSize(_width, _height)    => (),
        &HorizontalScrollBar(ref _visibility)   => (),
        &VerticalScrollBar(ref _visibility)     => (),
        &Fix(ref _axis)                         => ()
    }
}

///
/// Performs the actions associated with basic event registration for a widget
///
pub fn process_basic_event_request<W: GtkUiWidget>(widget: &W, flo_gtk: &mut FloGtk, event_type: GtkWidgetEventType, action_name: &String) {
    use self::GtkWidgetEventType::*;

    let action_name = action_name.clone();
    let event_sink  = flo_gtk.get_event_sink();

    match event_type {
        Click => {
            ClickActions::wire_widget(event_sink, widget, action_name.clone());
        },

        Paint(device) => {
            PaintActions::wire_widget(flo_gtk.widget_data(), event_sink, widget, action_name.clone(), device);
        },

        Drag => {
            DragActions::wire_widget(flo_gtk.widget_data(), event_sink, widget, action_name.clone());
        },

        VirtualScroll(_, _) | EditValue | SetValue | Dismiss => { }
    }
}

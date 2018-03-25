use super::layout::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

///
/// Performs the basic processing associated with a widget action (using a generic Gtk widget as the target)
/// 
pub fn process_basic_widget_action(id: WidgetId, widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
    use self::GtkWidgetAction::*;

    match action {
        &Layout(ref layout)         => process_basic_widget_layout(id, widget, flo_gtk, layout),
        &Content(ref content)       => process_basic_widget_content(widget, flo_gtk, content),
        &Appearance(ref appearance) => process_basic_widget_appearance(widget, flo_gtk, appearance),
        &State(ref state)           => process_basic_widget_state(widget, flo_gtk, state),
        &Font(ref font)             => process_basic_widget_font(widget, flo_gtk, font),

        &New(_widget_type)          => (),
        &SetRoot(window_id)         => { flo_gtk.get_window(window_id).map(|window| window.borrow_mut().set_root(flo_gtk, widget)); },
        &Delete                     => { widget.unparent(); }
    }
}

pub fn process_basic_widget_layout(id: WidgetId, widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, layout: &WidgetLayout) { 
    // Fetch or create the layout for this widget
    let widget_data     = flo_gtk.widget_data();
    let widget_layout   = widget_data.get_widget_data_or_insert(id, || Layout::new());

    // Update it with the content of the command
    widget_layout.map(move |widget_layout| widget_layout.update(layout));

    // Tell the parent of this widget it needs relayout
    widget.queue_resize();
}

pub fn process_basic_widget_content(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, content: &WidgetContent) {
    use self::WidgetContent::*;

    match content {
        &SetParent(parent_id)   => { widget.unparent(); flo_gtk.widget_data().get_widget(parent_id).map(|parent_widget| parent_widget.borrow_mut().add_child(widget)); },
        &SetText(ref _text)     => () /* Standard gtk widgets can't have text in them */,
        &Draw(ref canvas)       => unimplemented!()
    }
}

pub fn process_basic_widget_appearance(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, appearance: &Appearance) {
    use self::Appearance::*;

    match appearance {
        &Foreground(ref color)      => (),
        &Background(ref color)      => (),
        &Image(ref color)           => ()
    }
}

pub fn process_basic_widget_state(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, state: &State) {
    use self::State::*;

    match state {
        &Selected(ref selected_prop)    => (),
        &Badged(ref badged_prop)        => (),
        &Value(ref value_prop)          => (),
        &Range((ref from, ref to))      => ()
    }
}

pub fn process_basic_widget_font(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, font: &Font) {
    use self::Font::*;

    match font {
        &Size(size_pixels)      => (),
        &Align(ref align)       => (),
        &Weight(ref weight)     => ()
    }
}

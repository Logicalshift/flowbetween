use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

///
/// Performs the basic processing associated with a widget action (using a generic Gtk widget as the target)
/// 
pub fn process_basic_widget_action(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
    use self::GtkWidgetAction::*;

    match action {
        &Layout(ref layout)         => process_basic_widget_layout(widget, flo_gtk, layout),
        &Content(ref content)       => process_basic_widget_content(widget, flo_gtk, content),
        &Appearance(ref appearance) => process_basic_widget_appearance(widget, flo_gtk, appearance),
        &State(ref state)           => process_basic_widget_state(widget, flo_gtk, state),
        &Font(ref font)             => process_basic_widget_font(widget, flo_gtk, font),

        &New(_widget_type)          => (),
        &SetRoot(window_id)         => { flo_gtk.get_window(window_id).map(|window| window.borrow_mut().set_root(flo_gtk, widget)); },
        &Delete                     => { widget.unparent(); }
    }
}

pub fn process_basic_widget_layout(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, layout: &WidgetLayout) { 
    unimplemented!()
}

pub fn process_basic_widget_content(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, content: &WidgetContent) {
    use self::WidgetContent::*;

    match content {
        &SetParent(parent_id)   => { widget.unparent(); flo_gtk.get_widget(parent_id).map(|parent_widget| parent_widget.borrow_mut().add_child(widget)); },
        &SetText(ref _text)     => () /* Standard gtk widgets can't have text in them */,
        &Draw(ref canvas)       => unimplemented!()
    }
}

pub fn process_basic_widget_appearance(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, appearance: &Appearance) {
    unimplemented!()
}

pub fn process_basic_widget_state(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, state: &State) {
    unimplemented!()
}

pub fn process_basic_widget_font(widget: &mut gtk::Widget, flo_gtk: &mut FloGtk, font: &Font) {
    unimplemented!()
}

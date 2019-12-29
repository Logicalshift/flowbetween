use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// The label widget is used to manage GTK labels
///
pub struct FloLabelWidget {
    /// The ID of the widget
    id: WidgetId,

    /// The label as a label
    label: gtk::Label,

    /// The label as a widget
    widget: gtk::Widget
}

impl FloLabelWidget {
    ///
    /// Creates a basic widget
    ///
    pub fn new<Src: Cast+Clone+IsA<gtk::Widget>+IsA<gtk::Label>>(id: WidgetId, widget: Src) -> FloLabelWidget {
        // Fetch the object references
        let label   = widget.clone().upcast::<gtk::Label>();
        let widget  = widget.upcast::<gtk::Widget>();

        // FlowBetween labels are left-aligned by default
        label.set_xalign(0.0);

        // Generate the final widget
        FloLabelWidget {
            id:     id,
            label:  label,
            widget: widget
        }
    }
}

impl GtkUiWidget for FloLabelWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;
        use self::WidgetContent::*;
        use self::Font::*;

        match action {
            &Font(Align(TextAlign::Left))   => { self.label.set_xalign(0.0); },
            &Font(Align(TextAlign::Center)) => { self.label.set_xalign(0.5); },
            &Font(Align(TextAlign::Right))  => { self.label.set_xalign(1.0); },

            &Content(SetText(ref new_text)) => { self.label.set_text(&*new_text); },
            other_action                    => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // Labels have no child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.widget
    }
}

use super::widget::*;
use super::basic_widget::*;
use super::flo_layout::*;
use super::widget_data::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Represents the behaviour of a widget that can contain Flo content (such as labels, etc)
/// 
pub struct FloWidget {
    /// The ID assigned to this widget
    id: WidgetId,

    /// The widget that will contain the content for this widget
    container: gtk::Container,

    /// The widget interface for our container
    as_widget: gtk::Widget,

    /// The widget used to display the text for this itme
    text: Option<gtk::Label>,

    /// Used to lay out the content of the container
    layout: Rc<RefCell<FloWidgetLayout>>
}

impl FloWidget {
    ///
    /// Creates a new FloWidget that can contain generic controls
    /// 
    pub fn new(id: WidgetId, widget_data: Rc<WidgetData>) -> FloWidget {
        let fixed = gtk::Fixed::new();
            
        FloWidget {
            id:         id,
            container:  fixed.clone().upcast::<gtk::Container>(),
            as_widget:  fixed.clone().upcast::<gtk::Widget>(),
            text:       None,
            layout:     Rc::new(RefCell::new(FloWidgetLayout::new(widget_data)))
        }
    }

    ///
    /// Sets the text label for this widget
    /// 
    pub fn set_text(&mut self, new_text: &str) {
        // Get the label for this widget
        let container   = &mut self.container;
        let text_label  = self.text.get_or_insert_with(|| {
            let label = gtk::Label::new(new_text);
            container.add(&label);
            label.show_all();
            label
        });

        // Update the text within the label
        text_label.set_text(new_text);
    }
}

impl GtkUiWidget for FloWidget {
    ///
    /// Retrieves the ID assigned to this widget
    /// 
    fn id(&self) -> WidgetId {
        self.id
    }

    ///
    /// Processes an action for this widget
    ///
    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        match action {
            &GtkWidgetAction::Content(WidgetContent::SetText(ref new_text)) => {
                self.set_text(new_text);
            },

            // Any other action is processed as normal
            other => { process_basic_widget_action(self.id, &self.as_widget, flo_gtk, other); }
        }
    }

    ///
    /// Sets the children of this widget
    /// 
    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // Remove any existing children
        self.container.get_children().into_iter().for_each(|existing| self.container.remove(&existing));

        // Add children to this widget
        for child in children {
            self.container.add(child.borrow().get_underlying());
        }
    }

    ///
    /// Retrieves the underlying widget for this UI widget
    /// 
    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}
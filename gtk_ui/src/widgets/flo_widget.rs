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

    /// Widget data
    widget_data: Rc<WidgetData>,

    /// The IDs of the child widgets of this widget
    child_ids: Vec<WidgetId>,

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
    /// Creates a new FloWidget that can contain generic controls using the fixed layout style
    /// 
    pub fn new<Container: Cast+IsA<gtk::Container>>(id: WidgetId, container: Container, widget_data: Rc<WidgetData>) -> FloWidget {
        // Cast the container to a gtk container
        let container = container.upcast::<gtk::Container>();

        // Create the widget
        let layout  = Rc::new(RefCell::new(FloWidgetLayout::new(Rc::clone(&widget_data))));

        // Attach events to it
        Self::attach_layout_signal(&container.clone(), Rc::clone(&layout));
            
        // Build the final structure
        FloWidget {
            id:             id,
            widget_data:    widget_data,
            child_ids:      vec![],
            container:      container.clone(),
            as_widget:      container.upcast::<gtk::Widget>(),
            text:           None,
            layout:         layout
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

    ///
    /// Attaches a signal handler to perform layout in the specified container when resized
    /// 
    fn attach_layout_signal(container: &gtk::Container, layout: Rc<RefCell<FloWidgetLayout>>) {
        container.connect_size_allocate(move |container, _allocation| {
            layout.borrow().layout_fixed(container);
        });
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
            other => { process_basic_widget_action(self, flo_gtk, other); }
        }
    }

    ///
    /// Sets the children of this widget
    /// 
    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        let widget_data = &self.widget_data;
        let container   = &self.container;

        // Remove any child widgets added by the previous call to this function
        self.child_ids.drain(..)
            .map(|child_id| widget_data.get_widget(child_id))
            .for_each(|widget| { widget.map(|widget| container.remove(widget.borrow().get_underlying())); });

        // Send to the layout
        self.layout.borrow_mut().set_children(children.iter().map(|widget| widget.borrow().id()));

        // Add children to this widget
        self.child_ids.extend(children.iter().map(|child_widget| child_widget.borrow().id()));
        for child in children {
            self.container.add(child.borrow().get_underlying());
        }

        // Queue a resize so the layout is done
        self.layout.borrow().layout_fixed(&self.container);
        self.container.queue_resize();
    }

    ///
    /// Retrieves the underlying widget for this UI widget
    /// 
    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}
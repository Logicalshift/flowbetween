use super::image::*;
use super::widget::*;
use super::basic_widget::*;
use super::flo_layout::*;
use super::widget_data::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Represents the behaviour of a widget that can contain Flo content (such as labels, etc)
/// 
pub struct FloFixedWidget {
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

    /// The widget used to display the text for this item
    text: Option<gtk::Label>,
    
    /// The widget used to display the image for this item
    image: Option<gtk::Image>,

    /// Used to lay out the content of the container
    layout: Rc<RefCell<FloWidgetLayout>>
}

///
/// Trait used to describe how to perform layout in a fixed widget
/// 
pub trait FixedWidgetLayout {
    fn attach_layout_signal(widget: Self, layout: Rc<RefCell<FloWidgetLayout>>);
}

impl FloFixedWidget {
    ///
    /// Creates a new FloWidget that can contain generic controls using the fixed layout style
    /// 
    pub fn new<Container: Cast+Clone+IsA<gtk::Container>+FixedWidgetLayout>(id: WidgetId, container_widget: Container, widget_data: Rc<WidgetData>) -> FloFixedWidget {
        // Cast the container to a gtk container
        let container = container_widget.clone().upcast::<gtk::Container>();

        // Create the widget
        let layout  = Rc::new(RefCell::new(FloWidgetLayout::new(Rc::clone(&widget_data))));

        // Attach events to it
        Container::attach_layout_signal(container_widget, Rc::clone(&layout));
            
        // Build the final structure
        FloFixedWidget {
            id:             id,
            widget_data:    widget_data,
            child_ids:      vec![],
            container:      container.clone(),
            as_widget:      container.upcast::<gtk::Widget>(),
            text:           None,
            image:          None,
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
    /// Sets or removes the image for this widget
    /// 
    pub fn set_image(&mut self, new_image: Option<Resource<Image>>) {
        // We entirely replace the image widget every time
        let mut new_image_widget = None;

        if let Some(new_image) = new_image {
            // Create a new image widget
            let image_widget = image_from_image(new_image);

            // This is the image widget we'll put into this container
            new_image_widget    = Some(image_widget);
        }

        // Remove the previous image widget if there is one
        let container = &mut self.container;
        self.image.take().map(|old_image| container.remove(&old_image));
        
        // Add the new image widget if we created one
        self.image = new_image_widget;
        self.image.as_ref().map(|new_image| {
            container.add(new_image);
            new_image.show();
        });
    }
}

impl FixedWidgetLayout for gtk::Fixed {
    fn attach_layout_signal(fixed: gtk::Fixed, layout: Rc<RefCell<FloWidgetLayout>>) {
        let container = fixed.upcast::<gtk::Container>();

        container.connect_size_allocate(move |container, _allocation| {
            layout.borrow().layout_fixed(container);
        });
    }
}

impl FixedWidgetLayout for gtk::Layout {
    fn attach_layout_signal(layout_widget: gtk::Layout, layout: Rc<RefCell<FloWidgetLayout>>) {
        layout_widget.connect_size_allocate(move |layout_widget, _allocation| {
            layout.borrow().layout_in_layout(layout_widget);
        });
    }
}

impl GtkUiWidget for FloFixedWidget {
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

            &GtkWidgetAction::Appearance(Appearance::Image(ref image_data)) => {
                self.set_image(Some(image_data.clone()));
            },

            // Any other action is processed as normal
            other => { process_basic_widget_action(self, flo_gtk, other); }
        }
    }

    ///
    /// Sets the children of this widget
    /// 
    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        {
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
            for child in children.iter() {
                self.container.add(child.borrow().get_underlying());
            }
        }

        // Queue a resize so the layout is done
        self.container.queue_resize();
    }

    ///
    /// Retrieves the underlying widget for this UI widget
    /// 
    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

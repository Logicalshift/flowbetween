use super::image::*;
use super::widget::*;
use super::basic_widget::*;
use super::flo_layout::*;
use super::widget_data::*;
use super::proxy_widget::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;
use gdk_pixbuf;
use gdk_pixbuf::prelude::*;

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

impl FloFixedWidget {
    ///
    /// Creates a new FloWidget that can contain generic controls using the fixed layout style
    /// 
    pub fn new<Container: Cast+IsA<gtk::Container>>(id: WidgetId, container: Container, widget_data: Rc<WidgetData>) -> FloFixedWidget {
        // Cast the container to a gtk container
        let container = container.upcast::<gtk::Container>();

        // Create the widget
        let layout  = Rc::new(RefCell::new(FloWidgetLayout::new(Rc::clone(&widget_data))));

        // Attach events to it
        Self::attach_layout_signal(&container.clone(), Rc::clone(&layout));
            
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
            let pixbuf          = pixbuf_from_image(new_image);
            let image_widget    = gtk::Image::new();
            
            // GTK can't auto-scale images, so we'll do that ourselves
            image_widget.connect_size_allocate(move |image, allocation| {
                let image = image.clone();
                if let Some(image) = image.dynamic_cast::<gtk::Image>().ok() {
                    // Work out the scale ratio for the image (so we fit it into the control but keep the aspect ratio)
                    let (image_width, image_height)     = (pixbuf.get_width() as f64, pixbuf.get_height() as f64);
                    let (target_width, target_height)   = (allocation.width as f64, allocation.height as f64);
                    let (ratio_w, ratio_h)              = (target_width/image_width, target_height/image_height);
                    let ratio                           = ratio_w.min(ratio_h);

                    // Create a scaled image with that ratio
                    let (new_width, new_height)         = (image_width * ratio, image_height * ratio);
                    let (new_width, new_height)         = (new_width as i32, new_height as i32);

                    // Scale the image to fit
                    let scaled = pixbuf.scale_simple(new_width, new_height, gdk_pixbuf::InterpType::Bilinear);
                    scaled.map(|scaled| image.set_from_pixbuf(&scaled));
                }
            });

            // This is the image widget we'll put into this container
            new_image_widget    = Some(image_widget);
        }

        // Remove the previous image widget if there is one
        let container = &mut self.container;
        self.image.take().map(|old_image| container.remove(&old_image));
        
        // Add the new image widget if we created one
        self.image = new_image_widget;
        self.image.as_ref().map(|new_image| container.add(new_image));
    }

    ///
    /// Attaches a signal handler to perform layout in the specified container when resized
    /// 
    fn attach_layout_signal(container: &gtk::Container, layout: Rc<RefCell<FloWidgetLayout>>) {
        container.connect_size_allocate(move |container, _allocation| {
            layout.borrow().layout_fixed(container);
        });
    }

    ///
    /// Ensures that a child widget has an associated window, so it can be Z-ordered
    /// 
    fn ensure_window(&mut self, child_widget_id: WidgetId) {
        // Fetch the current widget with this ID
        if let Some(existing_widget) = self.widget_data.get_widget(child_widget_id) {
            // Nothing to do if it already has a window
            if existing_widget.borrow().get_underlying().get_window().is_some() {
                return;
            }

            // Create a clone of the widget object
            let widget = existing_widget.borrow().get_underlying().clone();

            // No window. Wrap in an event box, which always has its own window
            let event_box = gtk::EventBox::new();

            self.container.remove(&widget);
            event_box.add(&widget);

            self.container.add(&event_box);

            // Substitute a proxy widget
            let proxy_event_box = ProxyWidget::new(Rc::clone(&existing_widget), event_box);
            self.widget_data.replace_widget(child_widget_id, proxy_event_box);
        }
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

        // Give them windows (TODO: we really need to only do this when we're about to perform a layout and some widgets have z-indexes)
        for child in children.iter() {
            let id = child.borrow().id();
            self.ensure_window(id);
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

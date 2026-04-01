use super::image::*;
use super::widget::*;
use super::basic_widget::*;
use super::flo_layout::*;
use super::widget_data::*;
use super::scroll_size::*;
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

    /// Callback that forces a re-layout of the widget
    force_relayout_fn: Box<dyn Fn() -> ()>,

    /// Callback that handles a change in the widget's viewport
    viewport_changed_fn: Box<dyn Fn() -> ()>,

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
    /// Forces this widget to lay out in a particular area
    fn force_layout(widget: Self, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>);

    /// Attaches the layout signal to this widget
    fn attach_layout_signal(widget: Self, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>);

    /// Moves anything in the layout that's attached to the viewport
    fn viewport_changed(widget: Self, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>);
}

impl FloFixedWidget {
    ///
    /// Creates a new FloWidget that can contain generic controls using the fixed layout style
    ///
    pub fn new<Container: 'static+Cast+Clone+IsA<gtk::Container>+FixedWidgetLayout>(id: WidgetId, container_widget: Container, widget_data: Rc<WidgetData>) -> FloFixedWidget {
        // Cast the container to a gtk container
        let container   = container_widget.clone().upcast::<gtk::Container>();

        // Create the widget
        let layout      = Rc::new(RefCell::new(FloWidgetLayout::new(id, Rc::clone(&widget_data))));

        // Create the 'force relayout' function
        let force_relayout = {
            let container_widget    = container_widget.clone();
            let layout              = Rc::clone(&layout);
            let layout_data         = Rc::clone(&widget_data);

            move || {
                Container::force_layout(container_widget.clone(), layout.clone(), id, &layout_data);
            }
        };

        let viewport_changed = {
            let container_widget    = container_widget.clone();
            let layout              = Rc::clone(&layout);
            let layout_data         = Rc::clone(&widget_data);

            move || {
                Container::viewport_changed(container_widget.clone(), layout.clone(), id, &layout_data);
            }
        };

        // Attach events to it
        Container::attach_layout_signal(container_widget, Rc::clone(&layout), id, &widget_data);

        // Build the final structure
        FloFixedWidget {
            id:                     id,
            widget_data:            widget_data,
            force_relayout_fn:      Box::new(force_relayout),
            viewport_changed_fn:    Box::new(viewport_changed),
            child_ids:              vec![],
            container:              container.clone(),
            as_widget:              container.upcast::<gtk::Widget>(),
            text:                   None,
            image:                  None,
            layout:                 layout
        }
    }

    ///
    /// Forces a re-layout of the content of this widget
    ///
    pub fn force_relayout(&self) {
        let relayout = &self.force_relayout_fn;
        relayout();
    }

    ///
    /// Sets the text label for this widget
    ///
    pub fn set_text(&mut self, new_text: &str) {
        // Get the label for this widget
        let container   = &mut self.container;
        let text_label  = self.text.get_or_insert_with(|| {
            let label = gtk::Label::new(Some(new_text));
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

    ///
    /// Callback when the viewport of this widget changes (eg, because it's the layout widget for a scroll widget)
    ///
    pub fn viewport_changed(&self) {
        (self.viewport_changed_fn)();
    }
}

impl FixedWidgetLayout for gtk::Fixed {
    fn force_layout(fixed: gtk::Fixed, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>) {
        let mut allocation = fixed.get_allocation();

        if let Some(widget_layout) = widget_data.get_widget_data::<WidgetPosition>(widget_id) {
            // If the layout has already decided on a maximum width, don't use a larger width than this (though we do allow the widget to shrink its contents if necessary)
            let widget_layout   = widget_layout.borrow();
            let max_width       = (widget_layout.x2-widget_layout.x1).max(1.0);
            let max_height      = (widget_layout.y2-widget_layout.y1).max(1.0);
            let max_width       = max_width as i32;
            let max_height      = max_height as i32;

            if allocation.width > max_width     { allocation.width = max_width; }
            if allocation.height > max_height   { allocation.height = max_height; }
        }

        layout.borrow_mut().force_next_layout();
        layout.borrow_mut().layout_fixed(&fixed, allocation);
    }

    fn viewport_changed(layout_widget: gtk::Fixed, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>) {
        layout.borrow_mut().layout_in_viewport(&layout_widget, widget_id, widget_data);
    }

    fn attach_layout_signal(fixed: gtk::Fixed, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>) {
        let widget_data = widget_data.clone();

        fixed.connect_size_allocate(move |fixed, allocation| {
            let mut allocation = *allocation;

            if let Some(widget_layout) = widget_data.get_widget_data::<WidgetPosition>(widget_id) {
                // If the layout has already decided on a maximum width, don't use a larger width than this (though we do allow the widget to shrink its contents if necessary)
                let widget_layout   = widget_layout.borrow();
                let max_width       = (widget_layout.x2-widget_layout.x1).max(1.0);
                let max_height      = (widget_layout.y2-widget_layout.y1).max(1.0);
                let max_width       = max_width as i32;
                let max_height      = max_height as i32;

                if allocation.width > max_width     { allocation.width = max_width; }
                if allocation.height > max_height   { allocation.height = max_height; }
            }

            layout.borrow_mut().layout_fixed(fixed, allocation);
        });
    }
}

impl FixedWidgetLayout for gtk::Layout {
    fn force_layout(layout_widget: gtk::Layout, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>) {
        let allocation          = layout_widget.get_allocation();
        let mut layout_width    = allocation.width;
        let mut layout_height   = allocation.height;

        if let Some(scroll_size) = widget_data.get_widget_data::<ScrollSize>(widget_id) {
            let scroll_size = scroll_size.borrow();

            layout_width    = layout_width.max(scroll_size.width);
            layout_height   = layout_height.max(scroll_size.height);

            if layout_widget.get_size() != (layout_width as u32, layout_height as u32) {
                layout_widget.set_size(layout_width as u32, layout_height as u32);
            }
        } else if let Some(widget_layout) = widget_data.get_widget_data::<WidgetPosition>(widget_id) {
            let widget_layout =  widget_layout.borrow();

            layout_width        = layout_width.max(widget_layout.width() as i32);
            layout_height       = layout_height.max(widget_layout.height() as i32);

            if layout_widget.get_size() != (layout_width as u32, layout_height as u32) {
                layout_widget.set_size(layout_width as u32, layout_height as u32);
            }
        }

        layout.borrow_mut().force_next_layout();
        layout.borrow_mut().layout_in_layout(&layout_widget, (0, 0), (layout_width, layout_height));
    }

    fn viewport_changed(layout_widget: gtk::Layout, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>) {
        layout.borrow_mut().layout_in_viewport(&layout_widget, widget_id, widget_data);
    }

    fn attach_layout_signal(layout_widget: gtk::Layout, layout: Rc<RefCell<FloWidgetLayout>>, widget_id: WidgetId, widget_data: &Rc<WidgetData>) {
        let widget_data = Rc::clone(widget_data);

        layout_widget.connect_size_allocate(move |layout_widget, allocation| {
            let mut layout_width    = allocation.width;
            let mut layout_height   = allocation.height;

            if let Some(scroll_size) = widget_data.get_widget_data::<ScrollSize>(widget_id) {
                let scroll_size = scroll_size.borrow();

                layout_width    = layout_width.max(scroll_size.width);
                layout_height   = layout_height.max(scroll_size.height);

                if layout_widget.get_size() != (layout_width as u32, layout_height as u32) {
                    layout_widget.set_size(layout_width as u32, layout_height as u32);
                }
            } else if let Some(widget_layout) = widget_data.get_widget_data::<WidgetPosition>(widget_id) {
                let widget_layout =  widget_layout.borrow();

                layout_width        = layout_width.max(widget_layout.width() as i32);
                layout_height       = layout_height.max(widget_layout.height() as i32);

                if layout_widget.get_size() != (layout_width as u32, layout_height as u32) {
                    layout_widget.set_size(layout_width as u32, layout_height as u32);
                }
            }

            layout.borrow_mut().layout_in_layout(layout_widget, (0, 0), (layout_width, layout_height));
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
    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
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
        self.layout.borrow_mut().force_next_layout();
        self.container.queue_resize();
    }

    ///
    /// Retrieves the underlying widget for this UI widget
    ///
    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

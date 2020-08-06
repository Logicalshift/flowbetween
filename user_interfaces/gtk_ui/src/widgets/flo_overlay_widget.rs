use super::image::*;
use super::widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::flo_fixed_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui as ui;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Handles events for 'overlay' widgets that make it possible to draw widgets on top of each other
///
/// This has a single 'overlaid' widget and a container which obeys the usual FlowBetween layout
/// rules
///
pub struct FloOverlayWidget {
    /// The ID of this widget
    id: WidgetId,

    /// The overlay widget that this is managing
    as_overlay: gtk::Overlay,

    /// The overlay again, but cast to a widget
    as_widget: gtk::Widget,

    /// The fixed widget used to 
    layout: FloFixedWidget,

    /// The container that the fixed widet is ov
    overlaid_widget: gtk::Widget,

    /// The IDs of the child widgets of this widget
    child_ids: Vec<WidgetId>,

    /// The widget data
    widget_data: Rc<WidgetData>
}

impl FloOverlayWidget {
    ///
    /// Creates a new overlay widget. An overlay widget consists of an interior layout widget and a single drawing
    /// widget that we overlay on the top of: it's particularly useful for image, colour or label backgrounds for
    /// widgets.
    ///
    pub fn new<W, Container>(id: WidgetId, overlay_widget: W, container_widget: Container, widget_data: Rc<WidgetData>) -> FloOverlayWidget 
    where   W:          Clone+Cast+IsA<gtk::Overlay>+IsA<gtk::Widget>,
            Container:  'static+Cast+Clone+IsA<gtk::Container>+IsA<gtk::Widget>+FixedWidgetLayout {
        // Make sure the container is displayed
        container_widget.show();

        // Create the container
        let layout          = FloFixedWidget::new(id, container_widget, Rc::clone(&widget_data));

        // Create the initial overlaid widget
        let overlaid_widget = gtk::Fixed::new();

        // Put together the widget structure
        let overlay_widget  = overlay_widget.upcast::<gtk::Overlay>();

        overlay_widget.add(&overlaid_widget);
        overlay_widget.add_overlay(layout.get_underlying());

        overlay_widget.reorder_overlay(layout.get_underlying(), 2);

        // Create the overlay
        FloOverlayWidget {
            id:                 id,
            as_overlay:         overlay_widget.clone().upcast::<gtk::Overlay>(),
            as_widget:          overlay_widget.upcast::<gtk::Widget>(),
            layout:             layout,
            overlaid_widget:    overlaid_widget.upcast::<gtk::Widget>(),
            child_ids:          vec![],
            widget_data:        widget_data
        }
    }

    ///
    /// Updates the widget we use for the overlay
    ///
    pub fn set_overlaid_widget(&mut self, new_overlaid_widget: gtk::Widget) {
        // Remove both child widgets
        self.as_overlay.remove(&self.overlaid_widget);

        self.overlaid_widget = new_overlaid_widget;

        // Add the child widgets back again
        self.as_overlay.add(&self.overlaid_widget);
        self.overlaid_widget.show();
    }

    ///
    /// Sets the image for this widget
    ///
    fn set_image(&mut self, new_image: ui::Resource<ui::Image>, flo_gtk: &mut FloGtk) {
        let image_widget = image_from_image(new_image.clone());
        image_widget.show();

        self.set_overlaid_widget(image_widget.upcast());
    }
}

impl GtkUiWidget for FloOverlayWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;

        match action {
            // Can set a background image (which becomes the overlaid widget)
            &Appearance(flo_ui::Appearance::Image(ref new_image))   => { self.set_image(new_image.clone(), flo_gtk); }

            // Showing the overlay shows all the widgets
            &Show                   => {
                self.overlaid_widget.show();
                self.layout.process(flo_gtk, action);
                process_basic_widget_action(self, flo_gtk, action);
            },

            _                       => {
                process_basic_widget_action(self, flo_gtk, action);
            }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // The layout widget is responsible for the children of this overlay widget
        self.layout.set_children(children);
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

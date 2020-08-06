use super::image::*;
use super::widget::*;
use super::widget_data::*;
use super::basic_widget::*;
use super::flo_overlay_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui;
use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Handles events for 'bin' widgets (which are containers which can only contain a single subcontrol)
///
/// Flo doesn't have this concept itself. A bin widget can contain a label or an image directly but needs
/// to delegate to a fixed widget for more complicated tasks.
///
pub struct FloBinWidget {
    /// The ID of this widget
    id: WidgetId,

    /// The bin widget that this is managing
    bin: gtk::Bin,

    /// The bin again, but cast to a widget
    as_widget: gtk::Widget,

    /// The widget data
    widget_data: Rc<WidgetData>,

    /// If all we have is a label, this is that label
    label: Option<(String, gtk::Label)>,

    /// If all we have is an image, this is that image
    image: Option<(Resource<Image>, gtk::Image)>,

    /// If we have more child widgets, then we create a fixed widget to store them in
    overlay: Option<FloOverlayWidget>
}

impl FloBinWidget {
    ///
    /// Creates a new bin widget
    ///
    pub fn new<W: Clone+Cast+IsA<gtk::Bin>+IsA<gtk::Widget>>(id: WidgetId, bin_widget: W, widget_data: Rc<WidgetData>) -> FloBinWidget {
        FloBinWidget {
            id:             id,
            bin:            bin_widget.clone().upcast::<gtk::Bin>(),
            as_widget:      bin_widget.upcast::<gtk::Widget>(),
            widget_data:    widget_data,
            label:          None,
            image:          None,
            overlay:        None
        }
    }

    ///
    /// Creates the fixed widget, if it doesn't already exist
    ///
    fn make_fixed(&mut self, flo_gtk: &mut FloGtk) {
        if self.overlay.is_none() {
            // Create the fixed widget
            let fixed_widget    = gtk::Overlay::new();
            let mut fixed       = FloOverlayWidget::new(self.id, fixed_widget.clone(), gtk::Fixed::new(), Rc::clone(&self.widget_data));

            fixed_widget.show();

            // Add labels and images if necessary (also removing the widgets)
            if let Some((label_text, label_widget)) = self.label.take() {
                self.bin.remove(&label_widget);
                label_widget.show();
                fixed.set_overlaid_widget(label_widget.upcast());
            }

            if let Some((image_resource, image_widget)) = self.image.take() {
                self.bin.remove(&image_widget);
                image_widget.show();
                fixed.set_overlaid_widget(image_widget.upcast());
            }

            // The fixed widget becomes the child of this widget
            self.overlay = Some(fixed);
            self.bin.add(&fixed_widget);
        }
    }

    ///
    /// Sets the label for this widget
    ///
    fn set_label(&mut self, new_label: String, flo_gtk: &mut FloGtk) {
        if let Some((ref mut label_text, ref label_widget)) = self.label {
            // If there's already a label control, then update it
            label_widget.set_text(&new_label);
            *label_text = new_label;
        } else if self.image.is_none() && self.overlay.is_none() {
            // If there's no label widget, create one
            let label_widget = gtk::Label::new(Some(new_label.as_str()));
            label_widget.show();

            self.bin.add(&label_widget);
            self.label = Some((new_label, label_widget));
        } else {
            // Send the request to the fixed object underneath this one
            self.make_fixed(flo_gtk);
            self.overlay.as_mut().map(|fixed| fixed.process(flo_gtk, &WidgetContent::SetText(new_label).into()));
        }
    }

    ///
    /// Sets the label for this widget
    ///
    fn set_image(&mut self, new_image: Resource<Image>, flo_gtk: &mut FloGtk) {
        // Remove any existing image control
        if let Some((_old_image, old_image_widget)) = self.image.take() {
            self.bin.remove(&old_image_widget);
        }

        if self.label.is_none() && self.overlay.is_none() {
            // If there's no image widget, create one
            let image_widget = image_from_image(new_image.clone());
            image_widget.show();

            self.bin.add(&image_widget);
            self.image = Some((new_image, image_widget));
        } else {
            // Send the request to the fixed object underneath this one
            self.make_fixed(flo_gtk);
            self.overlay.as_mut().map(|fixed| fixed.process(flo_gtk, &Appearance::Image(new_image).into()));
        }
    }
}

impl GtkUiWidget for FloBinWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;

        // Some actions should always be processed against the proxy widget
        match action {
            // Adding child widgets always generates the fixed widget before proceeding
            &Content(WidgetContent::SetChildren(_))                 => {
                self.make_fixed(flo_gtk);
                process_basic_widget_action(self, flo_gtk, action);
            },

            // Images and text can act as the entire content of a bin
            &Content(WidgetContent::SetText(ref new_text))          => { self.set_label(new_text.clone(), flo_gtk); }
            &Appearance(flo_ui::Appearance::Image(ref new_image))   => { self.set_image(new_image.clone(), flo_gtk); }

            // Events should come to this widget and not to the underlying fixed widget
            &RequestEvent(_, _)     => { process_basic_widget_action(self, flo_gtk, action); },

            // General appearance should apply to this widget
            &Appearance(_)          => { process_basic_widget_action(self, flo_gtk, action); },
            &State(_)               => { process_basic_widget_action(self, flo_gtk, action); },

            // The bin widget should become the root, not its content
            &SetRoot(_)             => { process_basic_widget_action(self, flo_gtk, action); },

            // Showing the bin shows both this and the proxy widget
            &Show                   => {
                process_basic_widget_action(self, flo_gtk, action);
                self.overlay.as_mut().map(|fixed| fixed.process(flo_gtk, action));
            },

            // Deletions remove the bin widget and not the underlying one
            &Delete                 => { process_basic_widget_action(self, flo_gtk, action); },

            // Everything else is either passed to the fixed widget or processed via the normal steps
            _                       => {
                if let Some(ref mut fixed) = self.overlay {
                    fixed.process(flo_gtk, action);
                } else {
                    process_basic_widget_action(self, flo_gtk, action);
                }
            }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // Child widgets are always added to the fixed widget
        self.overlay.as_mut().map(move |fixed| fixed.set_children(children));
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

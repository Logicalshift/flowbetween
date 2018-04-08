use super::widget::*;
use super::widget_data::*;
use super::flo_fixed_widget::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// The scroll widget manages a layout widget in order to provide a scrollable region
/// 
pub struct FloScrollWidget {
    /// The ID of this widget
    id:             WidgetId,

    /// The grid containing the scroll bars
    grid:           gtk::Grid,

    /// The same widget, cast as a widget
    as_widget:      gtk::Widget,

    /// The horizontal scrollbar for this widget
    horiz_scroll:   gtk::Scrollbar,

    /// The vertical scrollbar for this widget
    vert_scroll:    gtk::Scrollbar,

    /// The layout, where the actual child controls go
    layout:         gtk::Layout,

    /// We delegate the actual layout tasks (along with things like setting the image and text) to FloFixedWidget
    fixed_widget:   FloFixedWidget
}

impl FloScrollWidget {
    ///
    /// Creates a new scroll widget
    ///
    pub fn new(id: WidgetId, grid: gtk::Grid, widget_data: Rc<WidgetData>) -> FloScrollWidget {
        // Create the widgets
        let layout          = gtk::Layout::new(None, None);
        let horiz_scroll    = gtk::Scrollbar::new(gtk::Orientation::Horizontal, None);
        let vert_scroll     = gtk::Scrollbar::new(gtk::Orientation::Vertical, None);
        let packer          = gtk::Fixed::new();

        // Set up the various widgets
        packer.set_size_request(0,0);
        layout.set_hexpand(true);
        layout.set_vexpand(true);

        horiz_scroll.set_adjustment(&layout.get_hadjustment().unwrap());
        vert_scroll.set_adjustment(&layout.get_vadjustment().unwrap());

        // Fill up the grid
        grid.attach(&layout, 0, 0, 1, 1);
        grid.attach(&horiz_scroll, 0, 1, 1, 1);
        grid.attach(&vert_scroll, 1, 0, 1, 1);
        grid.attach(&packer, 1, 1, 1, 1);

        let as_widget       = grid.clone().upcast::<gtk::Widget>();
        let fixed_widget    = FloFixedWidget::new(id, layout.clone(), widget_data);

        FloScrollWidget {
            id:             id,
            grid:           grid,
            horiz_scroll:   horiz_scroll,
            vert_scroll:    vert_scroll,
            layout:         layout,
            as_widget:      as_widget,
            fixed_widget:   fixed_widget
        }
    }
}

impl GtkUiWidget for FloScrollWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;
        use self::Scroll::*;

        match action {
            &Scroll(MinimumContentSize(width, height))  => { self.layout.set_size(width as u32, height as u32); },
            &Scroll(HorizontalScrollBar(visibility))    => { /* TODO */ },
            &Scroll(VerticalScrollBar(visibility))      => { /* TODO */ },

            // All other actions act as if the fixed widget performed them
            other_action => { self.fixed_widget.process(flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        self.fixed_widget.set_children(children);
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

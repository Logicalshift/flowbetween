use super::layout::*;
use super::widget_data::*;
use super::super::gtk_action::*;

use flo_ui::*;

use gtk;
use gtk_sys;
use gtk::prelude::*;
use glib::translate::ToGlibPtr;

use std::rc::*;
use std::collections::HashSet;

///
/// Provides the computed layout position for a widget
/// 
#[derive(Clone, Copy)]
pub struct WidgetPosition {
    id: WidgetId,

    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,

    padding: (u32, u32, u32, u32),
    z_index: u32
}

///
/// Manages layout of a set of child widgets according to the standard flo layout rules
/// 
pub struct FloWidgetLayout {
    /// The child widgets that need to be laid out (in the order that they should be laid out)
    child_widget_ids: Vec<WidgetId>,

    /// Our copy of the widget data
    widget_data: Rc<WidgetData>
}

impl FloWidgetLayout {
    ///
    /// Creates a new Gtk widget layout object
    /// 
    pub fn new(widget_data: Rc<WidgetData>) -> FloWidgetLayout {
        FloWidgetLayout {
            child_widget_ids:   vec![],
            widget_data:        widget_data
        }
    }

    ///
    /// Sets the ID of the child widgets that this will lay out
    /// 
    pub fn set_children<W: IntoIterator<Item=WidgetId>>(&mut self, widgets: W) {
        self.child_widget_ids = widgets.into_iter().collect();
    }

    ///
    /// Turns a Position into an absolute position
    /// 
    pub fn layout_position(&self, last_pos: f32, next_pos: &Position, max_pos: f32, stretch_area: f32, total_stretch: f32) -> f32 {
        use self::Position::*;

        match next_pos {
            &At(pos)                    => pos,
            &Floating(ref _prop, offset) => offset, /* TODO: need to layout via viewmodel */
            &Offset(offset)             => last_pos + offset,
            &Stretch(portion)           => last_pos + stretch_area * (portion/total_stretch),
            &Start                      => 0.0,
            &End                        => max_pos,
            &After                      => last_pos
        }
    }

    ///
    /// Returns the amount of stretch in a position
    /// 
    fn get_stretch(&self, pos: &Position) -> f32 {
        if let &Position::Stretch(portion) = pos {
            portion
        } else {
            0.0
        }
    }

    ///
    /// Performs layout of the widgets in this item
    /// 
    pub fn get_layout(&self, width: f32, height: f32) -> Vec<WidgetPosition> {
        // Where we are in the current layout
        let mut xpos    = 0.0;
        let mut ypos    = 0.0;

        let mut total_stretch_x = 0.0;
        let mut total_stretch_y = 0.0;

        // First pass: lay out the components with no stretch
        let mut positions = vec![];

        for widget_id in self.child_widget_ids.iter() {
            // Get the layout for this widget
            let layout = self.widget_data.get_widget_data::<Layout>(*widget_id);
            let layout = layout
                .map(|layout| layout.borrow().clone())
                .unwrap_or_else(|| Layout::new());

            // Most important part is the bounds
            let bounds = layout.bounds.unwrap_or(Bounds::fill_all());

            // Decide on the bounds for this element
            let x1 = self.layout_position(xpos, &bounds.x1, width, 0.0, 1.0);
            let x2 = self.layout_position(x1, &bounds.x2, width, 0.0, 1.0);
            let y1 = self.layout_position(ypos, &bounds.y1, height, 0.0, 1.0);
            let y2 = self.layout_position(y1, &bounds.y2, height, 0.0, 1.0);

            // Incorporate any stretch
            total_stretch_x += self.get_stretch(&bounds.x1);
            total_stretch_x += self.get_stretch(&bounds.x2);
            total_stretch_y += self.get_stretch(&bounds.y1);
            total_stretch_y += self.get_stretch(&bounds.y2);

            // Update the xpos and ypos for the next pass
            xpos = x2;
            ypos = y2;
        }

        // Second pass: layout with stretch and generate the final positions
        if total_stretch_x == 0.0 { total_stretch_x = 1.0; }
        if total_stretch_y == 0.0 { total_stretch_y = 1.0; }

        let stretch_x = width - xpos;
        let stretch_y = height - ypos;

        for widget_id in self.child_widget_ids.iter() {
            // Get the layout for this widget
            let layout = self.widget_data.get_widget_data::<Layout>(*widget_id);
            let layout = layout
                .map(|layout| layout.borrow().clone())
                .unwrap_or_else(|| Layout::new());

            // Most important part is the bounds
            let bounds  = layout.bounds.unwrap_or(Bounds::fill_all());
            let padding = layout.padding.unwrap_or((0,0,0,0));
            let z_index = layout.z_index.unwrap_or(0);

            // Decide on the bounds for this element
            let x1 = self.layout_position(xpos, &bounds.x1, width, stretch_x, total_stretch_x);
            let x2 = self.layout_position(x1, &bounds.x2, width, stretch_x, total_stretch_x);
            let y1 = self.layout_position(ypos, &bounds.y1, height, stretch_y, total_stretch_y);
            let y2 = self.layout_position(y1, &bounds.y2, height, stretch_y, total_stretch_y);

            // Add to the position
            positions.push(WidgetPosition {
                id: *widget_id,
                x1, x2, y1, y2,
                padding,
                z_index
            });

            // Update the xpos and ypos for the next pass
            xpos = x2;
            ypos = y2;
        }

        positions
    }

    ///
    /// Lays out the widgets in a particular container (with 'Fixed' semantics - ie, GtkFixed or GtkLayout)
    /// 
    pub fn layout_fixed(&self, target: &gtk::Container) {
        // Fetch the width and height of the target
        let width       = target.get_allocated_width();
        let height      = target.get_allocated_height();

        // Get the layout for this widget
        let layout      = self.get_layout(width as f32, height as f32);

        // Position each of the widgets
        let mut remaining: HashSet<_> = target.get_children().into_iter().collect();

        for widget_layout in layout {
            // Fetch the widget we're going to lay out
            let widget = self.widget_data.get_widget(widget_layout.id);

            // Store this layout data with the widget
            self.widget_data.set_widget_data(widget_layout.id, widget_layout);

            // If the widget exists, then position it according to its coordinates (and padding)
            if let Some(widget) = widget {
                // Get the position from the layout
                let (x1, y1, x2, y2)            = (widget_layout.x1 as f64, widget_layout.y1 as f64, widget_layout.x2 as f64, widget_layout.y2 as f64);
                let (left, top, right, bottom)  = (widget_layout.padding.0 as f64, widget_layout.padding.1 as f64, widget_layout.padding.2 as f64, widget_layout.padding.3 as f64);

                // Convert to x, y and width and height
                let x       = x1+left;
                let y       = y1+top;
                let width   = (x2-x1)-(left+right);
                let height  = (y2-y1)-(top+bottom);

                // Borrow the widget and set its properties
                let widget      = widget.borrow();
                let underlying  = widget.get_underlying();

                remaining.remove(underlying);

                // Unsafe code used here because the rust GTK bindings don't have gtk_container_child_set_property
                unsafe {
                    let x = x.floor() as i32;
                    let y = y.floor() as i32;

                    gtk_sys::gtk_container_child_set_property(target.to_glib_none().0, underlying.to_glib_none().0, "x".to_glib_none().0,  gtk::Value::from(&x).to_glib_none().0);
                    gtk_sys::gtk_container_child_set_property(target.to_glib_none().0, underlying.to_glib_none().0, "y".to_glib_none().0,  gtk::Value::from(&y).to_glib_none().0);
                }
                // Can also cast to Fixed or Layout but the above code will work on either

                //target.child_set_property(underlying, "x", &(x.floor() as i32)).unwrap();
                //target.child_set_property(underlying, "y", &(y.floor() as i32)).unwrap()

                // Send a size request to the widget if its width or height has changed
                let (new_width, new_height) = (width.floor().max(0.0) as i32, height.floor().max(0.0) as i32);
                let (old_width, old_height) = (underlying.get_allocated_width(), underlying.get_allocated_height());

                if new_width != old_width || new_height != old_height {
                    underlying.queue_resize();
                }
                
                // Resize the widget
                underlying.set_size_request(new_width, new_height);
            }
        }

        // Make any remaining widget fill the entire container
        for extra_widget in remaining {
            unsafe {
                gtk_sys::gtk_container_child_set_property(target.to_glib_none().0, extra_widget.to_glib_none().0, "x".to_glib_none().0,  gtk::Value::from(&0).to_glib_none().0);
                gtk_sys::gtk_container_child_set_property(target.to_glib_none().0, extra_widget.to_glib_none().0, "y".to_glib_none().0,  gtk::Value::from(&0).to_glib_none().0);
            }

            extra_widget.set_size_request(width, height);
        }
    }
}

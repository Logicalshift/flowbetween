use super::layout::*;
use super::widget_data::*;
use super::super::gtk_action::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;
use gdk::prelude::*;

use std::rc::*;
use std::collections::HashSet;

///
/// Indicates the floating position of a widget (used when laying it out again) 
/// 
pub struct FloatingPosition {
    pub x: f32,
    pub y: f32
}

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
            &At(pos)                        => pos,
            &Floating(ref _prop, offset)    => offset,
            &Offset(offset)                 => last_pos + offset,
            &Stretch(portion)               => last_pos + stretch_area * (portion/total_stretch),
            &Start                          => 0.0,
            &End                            => max_pos,
            &After                          => last_pos
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

        xpos = 0.0;
        ypos = 0.0;

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
    /// Given a set of pairs of widget IDs and indexes, orders the corresponding widgets by Z-Index
    /// 
    /// Widgets that do not have a window will not be ordered.
    /// 
    pub fn order_zindex<ZIndexes: IntoIterator<Item=(WidgetId, u32)>>(&self, indexes: ZIndexes) {
        // Order the widgets by z-index
        let mut ordered_zindexes:Vec<_> = indexes.into_iter().collect();
        ordered_zindexes.sort_by_key(|&(_widget, z_index)| z_index);

        // Raise the windows in order
        // TODO: consider avoiding raising if the windows are already ordered?
        for (widget_id, _) in ordered_zindexes {
            self.widget_data.get_widget(widget_id)
                .map(|widget| {
                    widget.borrow()
                        .get_underlying()
                        .get_window()
                        .map(|window| window.raise());
                });
        }
    }

    ///
    /// Lays out the widgets in a particular container (with 'Fixed' semantics - ie, GtkFixed or GtkLayout)
    /// 
    pub fn layout_fixed(&self, target: &gtk::Container) {
        let allocation  = target.get_allocation();

        self.layout_in_container(target, allocation.x, allocation.y, allocation.width, allocation.height);
    }

    ///
    /// Lays out the widgets in a gtk::Layout continue
    /// 
    pub fn layout_in_layout(&self, target: &gtk::Layout) {
        let (width, height) = target.get_size();

        self.layout_in_container(target, 0, 0, width as i32, height as i32)
    }

    ///
    /// Performs container layout with a particular width and height
    /// 
    fn layout_in_container<T: Cast+Clone+IsA<gtk::Container>+IsA<gtk::Widget>>(&self, target: &T, min_x: i32, min_y: i32, width: i32, height: i32) {
        // Get the layout for this widget
        let layout      = self.get_layout(width as f32, height as f32);

        // Position each of the widgets
        let mut remaining: HashSet<_>   = target.get_children().into_iter().collect();
        let mut z_indices               = vec![];

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
                let mut x   = x1+left;
                let mut y   = y1+top;
                let width   = (x2-x1)-(left+right);
                let height  = (y2-y1)-(top+bottom);

                // Adjust by the floating position if there is one (this will be value as it was last updated via the viewmodel)
                if let Some(floating) = self.widget_data.get_widget_data::<FloatingPosition>(widget_layout.id) {
                    let floating = floating.borrow();
                    x += floating.x as f64;
                    y += floating.y as f64;
                }

                // Borrow the widget and set its properties
                let widget      = widget.borrow();
                let underlying  = widget.get_underlying();

                remaining.remove(underlying);

                // Send a size request to the widget if its width or height has changed
                let (new_x, new_y)          = (x.floor() as i32, y.floor() as i32);
                let (new_x, new_y)          = (new_x + min_x, new_y + min_y);
                let (new_width, new_height) = (width.floor().max(0.0) as i32, height.floor().max(0.0) as i32);

                // Suppress a GTK warning
                let _preferred_size = (underlying.get_preferred_width(), underlying.get_preferred_height());    // Side-effect: suppress warning about fixed layout
                
                // Resize the widget
                let existing_allocation = underlying.get_allocation();
                let new_allocation      = gtk::Rectangle { x: new_x, y: new_y, width: new_width, height: new_height };

                if existing_allocation != new_allocation {
                    underlying.size_allocate(&mut gtk::Rectangle { x: new_x, y: new_y, width: new_width, height: new_height });
                }

                // Store the z-index for later ordering
                z_indices.push((widget_layout.id, widget_layout.z_index));
            }
        }

        // Order z-indices of the widgets we've just been through (assuming they have windows that can be ordered)
        self.order_zindex(z_indices);

        // Make any remaining widget fill the entire container
        for extra_widget in remaining {
            // Stop GTK moaning that we're doing fixed layout
            let _preferred_size = (extra_widget.get_preferred_width(), extra_widget.get_preferred_height());    // Side-effect: suppress warning about fixed layout

            // Allocate the size for this widget
            extra_widget.size_allocate(&mut gtk::Rectangle { x: min_x, y: min_y, width: width, height: height })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::*;
    use super::super::super::gtk_thread::*;

    use std::cell::*;

    impl GtkUiWidget for WidgetId {
        fn id(&self) -> WidgetId                                                { *self }
        fn process(&mut self, _flo_gtk: &mut FloGtk, _action: &GtkWidgetAction) { }
        fn set_children(&mut self, _children: Vec<Rc<RefCell<GtkUiWidget>>>)    { }
        fn get_underlying<'a>(&'a self) -> &'a gtk::Widget                      { unimplemented!() }
    }

    #[test]
    fn basic_layout() {
        use self::Position::*;

        // Simple top, middle, bottom layout (this is FlowBetween's basic layout)
        let top         = WidgetId::Assigned(0);
        let middle      = WidgetId::Assigned(1);
        let bottom      = WidgetId::Assigned(2);
        let widget_data = Rc::new(WidgetData::new());

        widget_data.register_widget(top, top);
        widget_data.register_widget(middle, middle);
        widget_data.register_widget(bottom, bottom);

        let top_bounds = Bounds {
            x1: Start,  y1: After,
            x2: End,    y2: Offset(32.0)
        };
        let middle_bounds = Bounds {
           x1: Start,   y1: After, 
           x2: End,     y2: Stretch(1.0)
        };
        let bottom_bounds = Bounds {
            x1: Start,  y1: After,
            x2: End,    y2: Offset(256.0)
        };

        widget_data.set_widget_data(top, Layout { bounds: Some(top_bounds), padding: None, z_index: None });
        widget_data.set_widget_data(middle, Layout { bounds: Some(middle_bounds), padding: None, z_index: None });
        widget_data.set_widget_data(bottom, Layout { bounds: Some(bottom_bounds), padding: None, z_index: None });

        // Create a layout for these bounds
        let mut layout = FloWidgetLayout::new(Rc::clone(&widget_data));
        layout.set_children(vec![ top, middle, bottom ]);

        // Perform the layout
        let new_layout = layout.get_layout(1920.0, 1080.0);

        // Check that when laid out in a specific area this produces the results we were expecting
        assert!(new_layout.len() == 3);

        assert!(new_layout[0].id == top);
        assert!(new_layout[0].x1 == 0.0);   assert!(new_layout[0].x2 == 1920.0);
        assert!(new_layout[0].y1 == 0.0);   assert!(new_layout[0].y2 == 32.0);

        assert!(new_layout[1].id == middle);
        assert!(new_layout[1].x1 == 0.0);   assert!(new_layout[1].x2 == 1920.0);
        assert!(new_layout[1].y1 == 32.0);  assert!(new_layout[1].y2 == 824.0);

        assert!(new_layout[2].id == bottom);
        assert!(new_layout[2].x1 == 0.0);   assert!(new_layout[2].x2 == 1920.0);
        assert!(new_layout[2].y1 == 824.0); assert!(new_layout[2].y2 == 1080.0);
    }
}
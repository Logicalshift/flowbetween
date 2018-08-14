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
    pub x: f64,
    pub y: f64
}

///
/// Provides the computed layout position for a widget
/// 
#[derive(Clone, Copy, Debug)]
pub struct WidgetPosition {
    pub id: WidgetId,

    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,

    pub z_index: u32
}

///
/// Manages layout of a set of child widgets according to the standard flo layout rules
/// 
pub struct FloWidgetLayout {
    /// The ID of the parent widget
    parent_widget_id: WidgetId,

    /// The child widgets that need to be laid out (in the order that they should be laid out)
    child_widget_ids: Vec<WidgetId>,

    /// Our copy of the widget data
    widget_data: Rc<WidgetData>
}

impl FloWidgetLayout {
    ///
    /// Creates a new Gtk widget layout object
    /// 
    pub fn new(parent_widget_id: WidgetId, widget_data: Rc<WidgetData>) -> FloWidgetLayout {
        FloWidgetLayout {
            parent_widget_id:   parent_widget_id,
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
    pub fn layout_position(&self, last_pos: f64, next_pos: &Position, max_pos: f64, stretch_area: f64, total_stretch: f64) -> f64 {
        use self::Position::*;

        match next_pos {
            &At(pos)                        => pos as f64,
            &Floating(ref _prop, offset)    => offset as f64,
            &Offset(offset)                 => last_pos + (offset as f64),
            &Stretch(portion)               => last_pos + stretch_area * ((portion as f64)/total_stretch),
            &Start                          => 0.0,
            &End                            => max_pos,
            &After                          => last_pos
        }
    }

    ///
    /// Returns the amount of stretch in a position
    /// 
    fn get_stretch(&self, pos: &Position) -> f64 {
        if let &Position::Stretch(portion) = pos {
            portion as f64
        } else {
            0.0
        }
    }

    ///
    /// Performs layout of the widgets in this item
    /// 
    pub fn get_layout(&self, width: f64, height: f64) -> Vec<WidgetPosition> {
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
    /// Retrieves the padding to use for the layout
    /// 
    fn get_padding(&self) -> ((i32, i32), (i32, i32)) {
        let layout                      = self.widget_data.get_widget_data::<Layout>(self.parent_widget_id);
        let padding                     = layout.map(|layout| layout.borrow().padding.unwrap_or((0,0,0,0))).unwrap_or((0,0,0,0));

        let (left, top, right, bottom)  = padding;
        ((left as i32, top as i32), (right as i32, bottom as i32))
    }

    ///
    /// Lays out the widgets in a particular container (with 'Fixed' semantics - ie, GtkFixed or GtkLayout)
    /// 
    pub fn layout_fixed(&self, target: &gtk::Container) {
        let ((left, top), (right, bottom))  = self.get_padding();
        let allocation                      = target.get_allocation();

        self.layout_in_container(target, allocation.x + left, allocation.y + top, allocation.width - (left+right), allocation.height - (top+bottom));
    }

    ///
    /// Lays out the widgets in a gtk::Layout continue
    /// 
    pub fn layout_in_layout(&self, target: &gtk::Layout) {
        let ((left, top), (right, bottom))  = self.get_padding();
        let (width, height)                 = target.get_size();

        self.layout_in_container(target, left, top, (width as i32) - (left+right), (height as i32) - (top+bottom));
    }

    ///
    /// Performs container layout with a particular width and height
    /// 
    fn layout_in_container<T: Cast+Clone+IsA<gtk::Container>+IsA<gtk::Widget>>(&self, target: &T, min_x: i32, min_y: i32, width: i32, height: i32) {
        // Get the layout for this widget
        let layout      = self.get_layout(width as f64, height as f64);

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

                // Convert to x, y and width and height
                let mut x   = x1;
                let mut y   = y1;
                let width   = x2-x1;
                let height  = y2-y1;

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
                
                // Resize the widget
                let existing_allocation = underlying.get_allocation();
                let mut new_allocation  = gtk::Rectangle { x: new_x, y: new_y, width: new_width, height: new_height };

                if existing_allocation != new_allocation {
                    // Make sure that the 'default' size is at least this big (so GTK won't shrink the widget)
                    underlying.set_size_request(new_allocation.width, new_allocation.height);

                    // Suppress a GTK warning
                    let _preferred_size = (underlying.get_preferred_width(), underlying.get_preferred_height());    // Side-effect: suppress warning about fixed layout

                    // Allocate the widget where we actually want it to go
                    underlying.size_allocate(&mut new_allocation);
                }

                // Store the z-index for later ordering
                z_indices.push((widget_layout.id, widget_layout.z_index));
            }
        }

        // Order z-indices of the widgets we've just been through (assuming they have windows that can be ordered)
        self.order_zindex(z_indices);

        // Make any remaining widget fill the entire container
        let full_size = gtk::Rectangle { x: min_x, y: min_y, width: width, height: height };
        for extra_widget in remaining {
            // Set the size request for this widget
            extra_widget.set_size_request(full_size.width, full_size.height);

            // Stop GTK moaning that we're doing fixed layout
            let _preferred_size = (extra_widget.get_preferred_width(), extra_widget.get_preferred_height());    // Side-effect: suppress warning about fixed layout

            // Allocate the size for this widget
            extra_widget.size_allocate(&mut full_size.clone());
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
use super::widget::*;
use super::widget_data::*;
use super::scroll_size::*;
use super::basic_widget::*;
use super::flo_fixed_widget::*;
use super::super::gtk_event::*;
use super::super::gtk_action::*;
use super::super::gtk_thread::*;
use super::super::gtk_event_parameter::*;
use super::super::gtk_widget_event_type::*;

use flo_ui::*;

use gtk;
use gtk::prelude::*;

use std::rc::*;
use std::cell::*;

///
/// Represents the current virtual scroll state of a widget
///
#[derive(Copy, Clone, PartialEq, Debug)]
struct VirtualScrollState {
    top_left:   (u32, u32),
    size:       (u32, u32)
}

///
/// The scroll widget manages a layout widget in order to provide a scrollable region
///
pub struct FloScrollWidget {
    /// The ID of this widget
    id:             WidgetId,

    /// The widget data object
    widget_data:    Rc<WidgetData>,

    /// The scrolling window
    scroll_window:  gtk::ScrolledWindow,

    /// The same, cast as a widget
    as_widget:      gtk::Widget,

    /// The layout, where the actual child controls go
    layout:         gtk::Layout,

    /// We delegate the actual layout tasks (along with things like setting the image and text) to FloFixedWidget
    fixed_widget:   Rc<RefCell<FloFixedWidget>>,

    /// The horizontal scrollbar policy
    h_policy:       gtk::PolicyType,

    /// The vertical scrollbar policy
    v_policy:       gtk::PolicyType,
}

impl FloScrollWidget {
    ///
    /// Creates a new scroll widget
    ///
    pub fn new(id: WidgetId, scroll_window: gtk::ScrolledWindow, widget_data: Rc<WidgetData>) -> FloScrollWidget {
        // Create the widgets
        let no_adjustment: Option<gtk::Adjustment> = None;
        let layout          = gtk::Layout::new(no_adjustment.as_ref(), no_adjustment.as_ref());

        // If the scroll window is created at 0 size, it generates a warning, so set a default min size to suppress it
        scroll_window.set_min_content_width(16);
        scroll_window.set_min_content_height(16);

        // Add the layout widget to the scroll widget
        scroll_window.set_policy(gtk::PolicyType::Always, gtk::PolicyType::Always);
        scroll_window.add(&layout);

        layout.show();

        // Generate the widget
        let as_widget       = scroll_window.clone().upcast::<gtk::Widget>();
        let fixed_widget    = FloFixedWidget::new(id, layout.clone(), widget_data.clone());
        let fixed_widget    = Rc::new(RefCell::new(fixed_widget));

        let widget = FloScrollWidget {
            id:             id,
            widget_data:    widget_data,
            scroll_window:  scroll_window,
            layout:         layout,
            as_widget:      as_widget,
            fixed_widget:   fixed_widget,
            h_policy:       gtk::PolicyType::Always,
            v_policy:       gtk::PolicyType::Always
        };

        widget.connect_viewport_changed_on_resize();
        widget.connect_viewport_changed_on_adjust(widget.layout.get_hadjustment().unwrap());
        widget.connect_viewport_changed_on_adjust(widget.layout.get_vadjustment().unwrap());

        widget
    }

    ///
    /// Tells the fixed widget that the viewport has changed when the scroll view has resized
    ///
    fn connect_viewport_changed_on_resize(&self) {
        let fixed_widget = Rc::downgrade(&self.fixed_widget);

        self.scroll_window.connect_size_allocate(move |_, _allocation| {
            if let Some(fixed_widget) = fixed_widget.upgrade() {
                fixed_widget.borrow().viewport_changed();
            }
        });
    }
    
    ///
    /// Tells the fixed widget that the viewport has changed when an adjustment changes
    ///
    fn connect_viewport_changed_on_adjust(&self, adjustment: gtk::Adjustment) {
        let fixed_widget = Rc::downgrade(&self.fixed_widget);

        adjustment.connect_value_changed(move |_| {
            if let Some(fixed_widget) = fixed_widget.upgrade() {
                fixed_widget.borrow().viewport_changed();
            }
        });
    }

    ///
    /// Generates the scrollbar visibility for a particular policy
    ///
    fn policy_for_visibility(visibility: ScrollBarVisibility) -> gtk::PolicyType {
        use self::ScrollBarVisibility::*;

        match visibility {
            Never           => gtk::PolicyType::Never,
            Always          => gtk::PolicyType::Always,
            OnlyIfNeeded    => gtk::PolicyType::Automatic
        }
    }

    ///
    /// Updates the policy for this scroll widget (which is what GTK calls the rules for showing the scroll bars)
    ///
    fn update_policy(&self) {
        self.scroll_window.set_policy(self.h_policy, self.v_policy);
    }

    ///
    /// Sends a virtual scroll event based on the current state of the widget to the specified event sink
    ///
    fn generate_virtual_scroll_event(widget_id: WidgetId, state: Rc<RefCell<VirtualScrollState>>, sink: &mut GtkEventSink, action_name: &str, layout: &gtk::Layout, width: f32, height: f32) {
        let width       = width as f64;
        let height      = height as f64;

        let h_adjust    = layout.get_hadjustment().unwrap();
        let v_adjust    = layout.get_vadjustment().unwrap();

        // Calculate the scroll position from the adjustments
        let page_x      = h_adjust.get_value();
        let page_y      = v_adjust.get_value();
        let page_w      = h_adjust.get_page_size();
        let page_h      = v_adjust.get_page_size();

        let grid_x      = page_x / width;
        let grid_y      = page_y / height;
        let grid_w      = page_w / width;
        let grid_h      = page_h / height;

        let grid_x      = grid_x.floor() as u32;
        let grid_y      = grid_y.floor() as u32;
        let grid_w      = (grid_w+0.5).ceil() as u32;
        let grid_h      = (grid_h+0.5).ceil() as u32;

        let mut old_state   = state.borrow_mut();
        let new_state       = VirtualScrollState {
            top_left:   (grid_x, grid_y),
            size:       (grid_w, grid_h)
        };

        // If the state changes, send the event
        if &*old_state != &new_state {
            // Store the state
            *old_state = new_state;

            // Send the event
            let scroll_parameter = GtkEventParameter::VirtualScroll((grid_x, grid_y), (grid_w, grid_h));
            publish_event(sink, GtkEvent::Event(widget_id, action_name.to_string(), scroll_parameter));
        }
    }

    ///
    /// Generates a virtual scroll event when the size allocation changes
    ///
    fn connect_virtual_scroll_on_resize(&self, state: Rc<RefCell<VirtualScrollState>>, sink: GtkEventSink, action_name: String, width: f32, height: f32) {
        let weak_layout = self.layout.clone().downgrade();
        let sink        = RefCell::new(sink);
        let widget_id   = self.id;

        // Generate a new virtual scroll event whenever the scroll window's size changes
        self.scroll_window.connect_size_allocate(move |_, _allocation| {
            if let Some(layout) = weak_layout.upgrade() {
                Self::generate_virtual_scroll_event(widget_id, Rc::clone(&state), &mut *sink.borrow_mut(), &action_name, &layout, width, height);
            }
        });
    }

    ///
    /// Updates the size of the scroll window content
    ///
    fn set_scroll_window_size(id: WidgetId, widget_data: &Rc<WidgetData>, min_size: (f64, f64)) {
        widget_data.set_widget_data(id, ScrollSize { width: min_size.0 as i32, height: min_size.1 as i32 });
    }

    ///
    /// Generates a virtual scroll event when an adjustment changes
    ///
    fn connect_virtual_scroll_on_adjust(&self, adjustment: gtk::Adjustment, state: Rc<RefCell<VirtualScrollState>>, sink: GtkEventSink, action_name: String, width: f32, height: f32) {
        let weak_layout = self.layout.clone().downgrade();
        let sink        = RefCell::new(sink);
        let widget_id   = self.id;

        // Generate a new virtual scroll event whenever the adjustment's value changes
        adjustment.connect_value_changed(move |_| {
            if let Some(layout) = weak_layout.upgrade() {
                Self::generate_virtual_scroll_event(widget_id, Rc::clone(&state), &mut *sink.borrow_mut(), &action_name, &layout, width, height);
            }
        });
    }

    ///
    /// Begins responding to virtual scrolling events
    ///
    fn start_virtual_scrolling(&self, sink: GtkEventSink, action_name: String, width: f32, height: f32) {
        let mut sink = sink;

        // The scroll state is used to avoid regenerating virtual scroll events (for example when the user moves horizontally and vertically simultaneously)
        let scroll_state = VirtualScrollState { top_left: (0,0), size: (0,0) };
        let scroll_state = Rc::new(RefCell::new(scroll_state));

        // Generate the initial event
        Self::generate_virtual_scroll_event(self.id, Rc::clone(&scroll_state), &mut sink, &action_name, &self.layout, width, height);

        // Generate virtual scroll events when the size of the scroll area changes
        self.connect_virtual_scroll_on_resize(Rc::clone(&scroll_state), sink.clone(), action_name.clone(), width, height);
        self.connect_virtual_scroll_on_adjust(self.layout.get_hadjustment().unwrap(), Rc::clone(&scroll_state), sink.clone(), action_name.clone(), width, height);
        self.connect_virtual_scroll_on_adjust(self.layout.get_vadjustment().unwrap(), Rc::clone(&scroll_state), sink, action_name, width, height);
    }
}

impl GtkUiWidget for FloScrollWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        use self::GtkWidgetAction::*;
        use self::Scroll::*;
        use self::WidgetContent::SetText;
        use self::Appearance::Image;

        match action {
            // Scroll actions are handled by this control
            &Scroll(MinimumContentSize(width, height))  => {
                Self::set_scroll_window_size(self.id, &self.widget_data, (width as f64, height as f64));
                self.fixed_widget.borrow().force_relayout();
            },
            &Scroll(HorizontalScrollBar(visibility))    => { self.h_policy = Self::policy_for_visibility(visibility); self.update_policy(); },
            &Scroll(VerticalScrollBar(visibility))      => { self.v_policy = Self::policy_for_visibility(visibility); self.update_policy(); },

            // Content actions are handled by the fixed widget
            &Content(SetText(_))                        => { self.fixed_widget.borrow_mut().process(flo_gtk, action); },
            &Appearance(Image(_))                       => { self.fixed_widget.borrow_mut().process(flo_gtk, action); },

            // This can generate virtual scroll events
            &RequestEvent(GtkWidgetEventType::VirtualScroll(width, height), ref name) => self.start_virtual_scrolling(flo_gtk.get_event_sink(), name.clone(), width, height),

            // All other actions are basic actions
            other_action                                => { process_basic_widget_action(self, flo_gtk, other_action); }
        }
    }

    fn set_children(&mut self, children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        self.fixed_widget.borrow_mut().set_children(children);
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

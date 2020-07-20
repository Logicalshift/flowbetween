use crate::gtk_action::*;
use crate::gtk_thread::*;
use crate::widgets::*;
use crate::widgets::basic_widget::*;

use gtk::prelude::*;

use std::cell::*;
use std::rc::*;

///
/// The GFX canvas widget is a canvas that renders via the GFX library
///
pub struct FloGfxCanvasWidget {
    // The ID of this widget
    id: WidgetId,

    /// The widget that the rest of the code will deal with
    as_widget: gtk::Widget
}

impl FloGfxCanvasWidget {
    ///
    /// Creates a new GFX canvas widget that renders to the specified GL area
    ///
    pub fn new_opengl<W: Clone+Cast+IsA<gtk::GLArea>>(widget_id: WidgetId, widget: W) -> FloGfxCanvasWidget {
        let id          = widget_id;
        let as_glarea   = widget.clone().upcast::<gtk::GLArea>();
        let as_widget   = as_glarea.clone().upcast::<gtk::Widget>();

        FloGfxCanvasWidget {
            id,
            as_widget
        }
    }
}

impl GtkUiWidget for FloGfxCanvasWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        match action {
            &GtkWidgetAction::Content(WidgetContent::Draw(ref drawing)) => { },
            other_action                                                => process_basic_widget_action(self, flo_gtk, other_action)
        }
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<dyn GtkUiWidget>>>) {
        // GFX widgets cannot have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}

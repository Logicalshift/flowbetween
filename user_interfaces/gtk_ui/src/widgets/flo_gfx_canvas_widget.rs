use crate::gtk_action::*;

use gtk::prelude::*;

///
/// The GFX canvas widget is a canvas that renders via the GFX library
///
pub struct FloGfxCanvasWidget {

}

impl FloGfxCanvasWidget {
    ///
    /// Creates a new GFX canvas widget that renders to the specified GL area
    ///
    pub fn new_opengl<W: Clone+Cast+IsA<gtk::GLArea>>(widget_id: WidgetId, widget: W) -> FloGfxCanvasWidget {
        FloGfxCanvasWidget {

        }
    }
}
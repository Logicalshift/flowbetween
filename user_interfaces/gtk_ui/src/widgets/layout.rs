use super::super::gtk_action::*;

use flo_ui::*;

///
/// Data associated with a widget used for describing how it will be laid out
///
#[derive(Clone, Debug)]
pub struct Layout {
    /// The bounding box of this widget
    pub bounds: Option<Bounds>,

    /// The padding for this widget
    pub padding: Option<(u32, u32, u32, u32)>,

    /// The Z-index for this widget
    pub z_index: Option<u32>
}

impl Layout {
    ///
    /// Creates a new widgetlayout object
    ///
    pub fn new() -> Layout {
        Layout {
            bounds:     None,
            padding:    None,
            z_index:    None
        }
    }

    ///
    /// Updates this layout by interpreting a WidgetLayout action
    ///
    pub fn update(&mut self, layout: &WidgetLayout) {
        use self::WidgetLayout::*;

        match layout {
            &BoundingBox(ref bounds)                => self.bounds = Some(bounds.clone()),
            &ZIndex(z_index)                        => self.z_index = Some(z_index),
            &Padding((left, top), (right, bottom))  => self.padding = Some((left, top, right, bottom)),
            &Floating(_, _)                         => ()
        }
    }
}

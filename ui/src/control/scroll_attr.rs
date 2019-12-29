use super::*;

use ::modifier::*;

///
/// Possible visibilities for the scrollbars
///
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ScrollBarVisibility {
    Never,
    Always,
    OnlyIfNeeded
}

///
/// Specifies a fixed axis
///
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FixedAxis {
    /// Fixed in position along the horizontal axis
    Horizontal,

    /// Fixed in position along the vertical axis
    Vertical,

    /// Fixed in position along both axes
    Both
}

///
/// Attributes representing the way a control scrolls its content
///
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Scroll {
    /// The size of the content of this scroll region
    ///
    /// This is a minimum size. If there are items placed outside this region, the scroll
    /// region will grow to accomodate them.
    ///
    /// If the control is larger than this size, then the bounds will be set to the
    /// overall size of the control.
    MinimumContentSize(f32, f32),

    /// Specifies the visibility of the horizontal scroll bar
    HorizontalScrollBar(ScrollBarVisibility),

    /// Specifies the visibility of the vertical scroll bar
    VerticalScrollBar(ScrollBarVisibility),

    /// Fixes the position of this element relative to its containing scroll region
    ///
    /// It will be laid out as normal but will not move when the region is scrolled
    Fix(FixedAxis)
}

impl Modifier<Control> for Scroll {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::ScrollAttr(self))
    }
}

impl<'a> Modifier<Control> for &'a Scroll {
    fn modify(self, control: &mut Control) {
        control.add_attribute(ControlAttribute::ScrollAttr(self.clone()))
    }
}

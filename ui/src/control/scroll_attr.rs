use super::*;

use modifier::*;

///
/// Possible visibilities for the scrollbars
/// 
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ScrollBarVisibility {
    Never,
    Always,
    OnlyIfNeeded    
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
    VerticalScrollBar(ScrollBarVisibility)
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

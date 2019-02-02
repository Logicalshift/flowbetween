use super::action::*;
use super::actions_from::*;

use flo_ui::*;

impl ActionsFrom<ViewAction> for ControlAttribute {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, mut bind_property: BindProperty) -> Vec<ViewAction> { 
        use self::ControlAttribute::*;

        match self {
            FontAttr(font_attr)                     => font_attr.actions_from(bind_property),
            StateAttr(state_attr)                   => state_attr.actions_from(bind_property),
            PopupAttr(popup_attr)                   => popup_attr.actions_from(bind_property),
            AppearanceAttr(appearance_attr)         => appearance_attr.actions_from(bind_property),
            ScrollAttr(scroll_attr)                 => scroll_attr.actions_from(bind_property),
            HintAttr(hint_attr)                     => hint_attr.actions_from(bind_property),

            BoundingBox(bounds)                     => vec![ViewAction::SetBounds(bounds.clone())],
            ZIndex(z_index)                         => vec![ViewAction::SetZIndex(*z_index as f64)],
            Padding((left, top), (right, bottom))   => vec![ViewAction::SetPadding(*left as f64, *top as f64, *right as f64, *bottom as f64)],
            Text(text_val)                          => vec![ViewAction::SetText(bind_property(text_val.clone()))],
            Id(id)                                  => vec![ViewAction::SetId(id.clone())],
            Controller(name)                        => vec![],
            Action(trigger, name)                   => event_actions(trigger, name),
            Canvas(canvas_resource)                 => vec![],

            SubComponents(_components)              => vec![]               // Handled separately by ViewState
        }
    }
}

///
/// Creates the actions required to request a specific event trigger
///
fn event_actions(trigger: &ActionTrigger, name: &String) -> Vec<ViewAction> {
    use self::ActionTrigger::*;
    use self::PaintDevice::*;
    use self::MouseButton::*;

    match trigger {
        Click                           => vec![ViewAction::RequestEvent(ViewEvent::Click, name.clone())],
        Dismiss                         => vec![],
        
        Paint(Mouse(Left))              => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::MouseLeft), name.clone())],
        Paint(Mouse(Middle))            => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::MouseMiddle), name.clone())],
        Paint(Mouse(Right))             => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::MouseRight), name.clone())],
        Paint(Pen)                      => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::Pen), name.clone())],
        Paint(Eraser)                   => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::Eraser), name.clone())],

        Paint(Touch)                    => vec![],
        Paint(Mouse(MouseButton::Other(_))) => vec![],
        Paint(PaintDevice::Other)       => vec![],

        Drag                            => vec![],
        Focused                         => vec![],
        EditValue                       => vec![],
        SetValue                        => vec![],
        CancelEdit                      => vec![],
        VirtualScroll(width, height)    => vec![ViewAction::RequestEvent(ViewEvent::VirtualScroll(*width as f64, *height as f64), name.clone())],
    }
}

impl ActionsFrom<ViewAction> for Font {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: BindProperty) -> Vec<ViewAction> {
        use self::Font::*;

        match self {
            Size(size)      => vec![ViewAction::SetFontSize(*size as f64)],
            Align(align)    => vec![ViewAction::SetTextAlignment(*align)],
            Weight(weight)  => vec![ViewAction::SetFontWeight(*weight as u32 as f64)]
        }
    }
}

impl ActionsFrom<ViewAction> for State {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: BindProperty) -> Vec<ViewAction> {
        use self::State::*;

        match self {
            Selected(property)          => vec![],
            Badged(property)            => vec![],
            Enabled(property)           => vec![],
            Value(property)             => vec![],
            Range((lower, upper))       => vec![],
            FocusPriority(property)     => vec![]
        }
    }
}

impl ActionsFrom<ViewAction> for Popup {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: BindProperty) -> Vec<ViewAction> {
        use self::Popup::*;

        match self {
            IsOpen(property)        => vec![],
            Direction(direction)    => vec![],
            Size(width, height)     => vec![],
            Offset(offset)          => vec![]
        }
    }
}

impl ActionsFrom<ViewAction> for Appearance {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: BindProperty) -> Vec<ViewAction> {
        use self::Appearance::*;

        match self {
            Foreground(color)       => vec![ViewAction::SetForegroundColor(*color)],
            Background(color)       => vec![ViewAction::SetBackgroundColor(*color)],
            Image(image)            => vec![ViewAction::SetImage(image.clone())]
        }
    }
}

impl ActionsFrom<ViewAction> for Scroll {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: BindProperty) -> Vec<ViewAction> {
        use self::Scroll::*;

        match self {
            MinimumContentSize(width, height)   => vec![ViewAction::SetScrollMinimumSize(*width as f64, *height as f64)],
            HorizontalScrollBar(visibility)     => vec![ViewAction::SetHorizontalScrollBar(*visibility)],
            VerticalScrollBar(visibility)       => vec![ViewAction::SetVerticalScrollBar(*visibility)],
            Fix(axis)                           => vec![]
        }
    }
}

impl ActionsFrom<ViewAction> for Hint {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: BindProperty) -> Vec<ViewAction> {
        use self::Hint::*;

        match self {
            FastDrawing     => vec![],
            Class(name)     => vec![]
        }
    }
}

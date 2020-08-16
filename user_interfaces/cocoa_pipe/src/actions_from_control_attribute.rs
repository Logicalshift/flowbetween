use super::action::*;
use super::actions_from::*;

use flo_ui::*;

fn make_app_position<BindProperty: FnMut(Property) -> AppProperty>(ui_pos: &Position, bind_property: &mut BindProperty) -> AppPosition {
    use self::Position::*;

    match ui_pos {
        At(pos)                     => AppPosition::At(*pos as f64),
        Floating(property, pos)     => AppPosition::Floating(bind_property(property.clone()), *pos as f64),
        Offset(pos)                 => AppPosition::Offset(*pos as f64),
        Stretch(pos)                => AppPosition::Stretch(*pos as f64),
        Start                       => AppPosition::Start,
        End                         => AppPosition::End,
        After                       => AppPosition::After
    }
}

fn make_app_bounds<BindProperty: FnMut(Property) -> AppProperty>(ui_bounds: &Bounds, bind_property: &mut BindProperty) -> AppBounds {
    AppBounds {
        x1: make_app_position(&ui_bounds.x1, bind_property),
        y1: make_app_position(&ui_bounds.y1, bind_property),
        x2: make_app_position(&ui_bounds.x2, bind_property),
        y2: make_app_position(&ui_bounds.y2, bind_property)
    }
}

impl ActionsFrom<ViewAction> for ControlAttribute {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: &mut BindProperty) -> Vec<ViewAction> {
        use self::ControlAttribute::*;

        match self {
            FontAttr(font_attr)                     => font_attr.actions_from(bind_property),
            StateAttr(state_attr)                   => state_attr.actions_from(bind_property),
            PopupAttr(popup_attr)                   => popup_attr.actions_from(bind_property),
            AppearanceAttr(appearance_attr)         => appearance_attr.actions_from(bind_property),
            ScrollAttr(scroll_attr)                 => scroll_attr.actions_from(bind_property),
            HintAttr(hint_attr)                     => hint_attr.actions_from(bind_property),

            BoundingBox(bounds)                     => vec![ViewAction::SetBounds(make_app_bounds(bounds, bind_property))],
            ZIndex(z_index)                         => vec![ViewAction::SetZIndex(*z_index as f64)],
            Padding((left, top), (right, bottom))   => vec![ViewAction::SetPadding(*left as f64, *top as f64, *right as f64, *bottom as f64)],
            Text(text_val)                          => vec![ViewAction::SetText(bind_property(text_val.clone()))],
            Id(id)                                  => vec![ViewAction::SetId(id.clone())],
            Controller(_name)                       => vec![],
            Action(trigger, name)                   => event_actions(trigger, name),

            Canvas(_canvas_resource)                => vec![],              // Can send the whole canvas here, but more consistent if it's done in the same place it's attached

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
        Dismiss                         => vec![ViewAction::RequestEvent(ViewEvent::Dismiss, name.clone())],

        Paint(Mouse(Left))              => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::MouseLeft), name.clone())],
        Paint(Mouse(Middle))            => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::MouseMiddle), name.clone())],
        Paint(Mouse(Right))             => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::MouseRight), name.clone())],
        Paint(Pen)                      => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::Pen), name.clone())],
        Paint(Eraser)                   => vec![ViewAction::RequestEvent(ViewEvent::Paint(AppPaintDevice::Eraser), name.clone())],

        Paint(Touch)                    => vec![],
        Paint(Mouse(MouseButton::Other(_))) => vec![],
        Paint(PaintDevice::Other)       => vec![],

        Drag                            => vec![ViewAction::RequestEvent(ViewEvent::Drag, name.clone())],
        Focused                         => vec![ViewAction::RequestEvent(ViewEvent::Focused, name.clone())],
        EditValue                       => vec![ViewAction::RequestEvent(ViewEvent::EditValue, name.clone())],
        SetValue                        => vec![ViewAction::RequestEvent(ViewEvent::SetValue, name.clone())],
        CancelEdit                      => vec![ViewAction::RequestEvent(ViewEvent::CancelEdit, name.clone())],
        VirtualScroll(width, height)    => vec![ViewAction::RequestEvent(ViewEvent::VirtualScroll(*width as f64, *height as f64), name.clone())],
    }
}

impl ActionsFrom<ViewAction> for Font {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, _bind_property: &mut BindProperty) -> Vec<ViewAction> {
        use self::Font::*;

        match self {
            Size(size)      => vec![ViewAction::SetFontSize(*size as f64)],
            Align(align)    => vec![ViewAction::SetTextAlignment(*align)],
            Weight(weight)  => vec![ViewAction::SetFontWeight(*weight as u32 as f64)]
        }
    }
}

impl ActionsFrom<ViewAction> for State {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: &mut BindProperty) -> Vec<ViewAction> {
        use self::State::*;

        match self {
            Selected(property)          => vec![ViewAction::SetState(ViewStateUpdate::Selected(bind_property(property.clone())))],
            Badged(property)            => vec![ViewAction::SetState(ViewStateUpdate::Badged(bind_property(property.clone())))],
            Enabled(property)           => vec![ViewAction::SetState(ViewStateUpdate::Enabled(bind_property(property.clone())))],
            Value(property)             => vec![ViewAction::SetState(ViewStateUpdate::Value(bind_property(property.clone())))],
            Range((lower, upper))       => vec![ViewAction::SetState(ViewStateUpdate::Range(bind_property(lower.clone()), bind_property(upper.clone())))],
            FocusPriority(property)     => vec![ViewAction::SetState(ViewStateUpdate::FocusPriority(bind_property(property.clone())))]
        }
    }
}

impl ActionsFrom<ViewAction> for Popup {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: &mut BindProperty) -> Vec<ViewAction> {
        use self::Popup::*;

        match self {
            IsOpen(property)        => vec![ViewAction::Popup(ViewPopupAction::Open(bind_property(property.clone())))],
            Direction(direction)    => vec![ViewAction::Popup(ViewPopupAction::SetDirection(*direction))],
            Size(width, height)     => vec![ViewAction::Popup(ViewPopupAction::SetSize(*width as f64, *height as f64))],
            Offset(offset)          => vec![ViewAction::Popup(ViewPopupAction::SetOffset(*offset as f64))]
        }
    }
}

impl ActionsFrom<ViewAction> for Appearance {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, _bind_property: &mut BindProperty) -> Vec<ViewAction> {
        use self::Appearance::*;

        match self {
            Foreground(color)       => vec![ViewAction::SetForegroundColor(*color)],
            Background(color)       => vec![ViewAction::SetBackgroundColor(*color)],
            Image(image)            => vec![ViewAction::SetImage(image.clone())]
        }
    }
}

impl ActionsFrom<ViewAction> for Scroll {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, _bind_property: &mut BindProperty) -> Vec<ViewAction> {
        use self::Scroll::*;

        match self {
            MinimumContentSize(width, height)   => vec![ViewAction::SetScrollMinimumSize(*width as f64, *height as f64)],
            HorizontalScrollBar(visibility)     => vec![ViewAction::SetHorizontalScrollBar(*visibility)],
            VerticalScrollBar(visibility)       => vec![ViewAction::SetVerticalScrollBar(*visibility)],
            Fix(axis)                           => vec![ViewAction::SetState(ViewStateUpdate::FixScrollAxis(*axis))]
        }
    }
}

impl ActionsFrom<ViewAction> for Hint {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, _bind_property: &mut BindProperty) -> Vec<ViewAction> {
        use self::Hint::*;

        match self {
            FastDrawing     => vec![],
            Class(name)     => vec![ViewAction::SetState(ViewStateUpdate::AddClass(name.clone()))]
        }
    }
}

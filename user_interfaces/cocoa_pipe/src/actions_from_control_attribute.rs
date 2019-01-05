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
            Padding((left, top), (right, bottom))   => vec![],
            Text(text_val)                          => vec![ViewAction::SetText(bind_property(text_val.clone()))],
            Id(id)                                  => vec![],
            Controller(name)                        => vec![],
            Action(trigger, name)                   => vec![],
            Canvas(canvas_resource)                 => vec![],

            SubComponents(_components)              => vec![]               // Handled separately by ViewState
        }
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
            Image(image)            => vec![]
        }
    }
}

impl ActionsFrom<ViewAction> for Scroll {
    fn actions_from<BindProperty: FnMut(Property) -> AppProperty>(&self, bind_property: BindProperty) -> Vec<ViewAction> {
        use self::Scroll::*;

        match self {
            MinimumContentSize(width, height)   => vec![],
            HorizontalScrollBar(visibility)     => vec![],
            VerticalScrollBar(visibility)       => vec![],
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

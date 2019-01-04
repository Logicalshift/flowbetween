use super::action::*;
use super::actions_from::*;

use flo_ui::*;

impl ActionsFrom<ViewAction> for ControlAttribute {
    fn actions_from(&self) -> Vec<ViewAction> { 
        use self::ControlAttribute::*;

        match self {
            BoundingBox(bounds)                     => vec![ViewAction::SetBounds(bounds.clone())],
            ZIndex(z_index)                         => vec![ViewAction::SetZIndex(*z_index as f64)],
            Padding((left, top), (right, bottom))   => vec![],
            Text(text_val)                          => vec![],
            FontAttr(font_attr)                     => font_attr.actions_from(),
            StateAttr(state_attr)                   => state_attr.actions_from(),
            PopupAttr(popup_attr)                   => popup_attr.actions_from(),
            AppearanceAttr(appearance_attr)         => appearance_attr.actions_from(),
            ScrollAttr(scroll_attr)                 => scroll_attr.actions_from(),
            HintAttr(hint_attr)                     => hint_attr.actions_from(),
            Id(id)                                  => vec![],
            Controller(name)                        => vec![],
            Action(trigger, name)                   => vec![],
            Canvas(canvas_resource)                 => vec![],

            SubComponents(_components)              => vec![]               // Handled separately by ViewState
        }
    }
}

impl ActionsFrom<ViewAction> for Font {
    fn actions_from(&self) -> Vec<ViewAction> {
        vec![]
    }
}

impl ActionsFrom<ViewAction> for State {
    fn actions_from(&self) -> Vec<ViewAction> {
        vec![]
    }
}

impl ActionsFrom<ViewAction> for Popup {
    fn actions_from(&self) -> Vec<ViewAction> {
        vec![]
    }
}

impl ActionsFrom<ViewAction> for Appearance {
    fn actions_from(&self) -> Vec<ViewAction> {
        vec![]
    }
}

impl ActionsFrom<ViewAction> for Scroll {
    fn actions_from(&self) -> Vec<ViewAction> {
        vec![]
    }
}

impl ActionsFrom<ViewAction> for Hint {
    fn actions_from(&self) -> Vec<ViewAction> {
        vec![]
    }
}

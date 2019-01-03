use super::action::*;
use super::actions_from::*;

use flo_ui::*;

impl ActionsFrom<ViewAction> for ControlAttribute {
    fn actions_from(&self) -> Vec<ViewAction> { 
        use self::ControlAttribute::*;

        match self {
            BoundingBox(bounds)                     => vec![ViewAction::SetBounds(bounds.clone())],
            ZIndex(z_index)                         => vec![],
            Padding((left, top), (right, bottom))   => vec![],
            Text(text_val)                          => vec![],
            FontAttr(font_attr)                     => vec![],
            StateAttr(state_attr)                   => vec![],
            PopupAttr(popup_attr)                   => vec![],
            AppearanceAttr(appearance_attr)         => vec![],
            ScrollAttr(scroll_attr)                 => vec![],
            HintAttr(hint_attr)                     => vec![],
            Id(id)                                  => vec![],
            Controller(name)                        => vec![],
            Action(trigger, name)                   => vec![],
            Canvas(canvas_resource)                 => vec![],

            SubComponents(_components)              => vec![]               // Handled separately by ViewState
        }
    }
}

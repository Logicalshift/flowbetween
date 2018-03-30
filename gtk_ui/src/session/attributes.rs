use super::super::gtk_action::*;

use flo_ui::*;

///
/// Trait implemented by things that can be converted to GTK widget actions
/// 
pub trait ToGtkActions {
    ///
    /// Converts this itme to a set of GtkWIdgetActions required to render it to a GTK widget
    /// 
    fn to_gtk_actions(&self) -> Vec<GtkWidgetAction>;
}

impl ToGtkActions for ControlAttribute {
    fn to_gtk_actions(&self) -> Vec<GtkWidgetAction> {
        use self::ControlAttribute::*;

        match self {
            &BoundingBox(ref bounds)                => vec![ GtkWidgetAction::Layout(WidgetLayout::BoundingBox(bounds.clone())) ],
            &ZIndex(zindex)                         => vec![ GtkWidgetAction::Layout(WidgetLayout::ZIndex(zindex)) ],
            &Padding((left, top), (right, bottom))  => vec![ GtkWidgetAction::Layout(WidgetLayout::Padding((left, top), (right, bottom))) ],
            &Text(ref text)                         => unimplemented!(),
            &FontAttr(ref font)                     => unimplemented!(),
            &StateAttr(ref state)                   => unimplemented!(),
            &PopupAttr(ref popup)                   => unimplemented!(),
            &AppearanceAttr(ref appearance)         => unimplemented!(),
            &ScrollAttr(ref scroll)                 => unimplemented!(),
            &Id(ref id)                             => unimplemented!(),
            &SubComponents(ref _components)         => unimplemented!(),
            &Controller(ref controller_name)        => unimplemented!(),
            &Action(ref trigger, ref action_name)   => unimplemented!(),
            &Canvas(ref canvas)                     => unimplemented!()
        }
    }
}
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

impl<'a> From<&'a ControlAttribute> for GtkWidgetAction {
    fn from(attr: &'a ControlAttribute) -> GtkWidgetAction {
        use self::ControlAttribute::*;

        match attr {
            &BoundingBox(ref bounds)                => unimplemented!(),
            &ZIndex(zindex)                         => unimplemented!(),
            &Padding((left, top), (right, bottom))  => unimplemented!(),
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
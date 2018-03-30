use super::property_action::*;
use super::super::gtk_action::*;

use flo_ui::*;

pub type PropertyWidgetAction = PropertyAction<GtkWidgetAction>;

///
/// Trait implemented by things that can be converted to GTK widget actions
/// 
pub trait ToGtkActions {
    ///
    /// Converts this itme to a set of GtkWIdgetActions required to render it to a GTK widget
    /// 
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction>;
}

impl ToGtkActions for ControlAttribute {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::ControlAttribute::*;

        match self {
            &BoundingBox(ref bounds)                => vec![ GtkWidgetAction::Layout(WidgetLayout::BoundingBox(bounds.clone())) ].into_actions(),
            &ZIndex(zindex)                         => vec![ GtkWidgetAction::Layout(WidgetLayout::ZIndex(zindex)) ].into_actions(),
            &Padding((left, top), (right, bottom))  => vec![ GtkWidgetAction::Layout(WidgetLayout::Padding((left, top), (right, bottom))) ].into_actions(),
            
            &Text(ref text)                         => vec![ PropertyAction::from_property(text.clone(), |text| GtkWidgetAction::Content(WidgetContent::SetText(text.to_string()))) ],

            &FontAttr(ref font)                     => unimplemented!(),
            &StateAttr(ref state)                   => unimplemented!(),
            &PopupAttr(ref popup)                   => unimplemented!(),
            &AppearanceAttr(ref appearance)         => unimplemented!(),
            &ScrollAttr(ref scroll)                 => unimplemented!(),

            &Id(ref id)                             => unimplemented!(),
            &Action(ref trigger, ref action_name)   => unimplemented!(),
            &Canvas(ref canvas)                     => unimplemented!(),

            // The GTK layout doesn't need to know the controller
            &Controller(ref _controller_name)       => vec![],

            // Subcomponents are added elsewhere: we don't assign them here
            &SubComponents(ref _components)         => vec![]
        }
    }
}
use flo_ui::*;

///
/// Represents an action that can optionally be bound to an action
///
pub enum PropertyAction<Action> {
    Unbound(Action),
    Bound(Property, Box<dyn Fn(PropertyValue) -> Vec<Action>>)
}

impl<Action> PropertyAction<Action> {
    ///
    /// Converts a property into a PropertyAction via a function that takes a concrete binding and returns the action
    /// that should be produced for that value
    ///
    pub fn from_property<TFn: 'static+Fn(PropertyValue) -> Vec<Action>>(property: Property, convert_value: TFn) -> PropertyAction<Action> {
        PropertyAction::Bound(property, Box::new(convert_value))
    }
}

///
/// Convenience trait for converting things into lists of property actions
///
pub trait IntoPropertyActions<Action> {
    fn into_actions(self) -> Vec<PropertyAction<Action>>;
}

impl<Action> From<Action> for PropertyAction<Action> {
    fn from(action: Action) -> PropertyAction<Action> {
        PropertyAction::Unbound(action)
    }
}

impl<Action> IntoPropertyActions<Action> for Vec<Action> {
    fn into_actions(self) -> Vec<PropertyAction<Action>> {
        self.into_iter()
            .map(|action| action.into())
            .collect()
    }
}

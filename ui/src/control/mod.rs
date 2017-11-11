mod types;
mod bounds;
mod actions;
mod control;
mod position;
mod attributes;

pub use self::types::*;
pub use self::bounds::*;
pub use self::actions::*;
pub use self::control::*;
pub use self::position::*;
pub use self::attributes::*;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_button() {
        let button = Control::button();

        assert!(button.control_type() == ControlType::Button);
    }

    #[test]
    fn can_create_label_with_text() {
        let label = Control::label().with("Hello");

        assert!(label.control_type() == ControlType::Label);
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::Text("Hello".to_property())));
    }

    #[test]
    fn can_create_label_with_many_attributes() {
        let label = Control::label().with(("Hello", Bounds::fill_all()));

        assert!(label.control_type() == ControlType::Label);
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::Text("Hello".to_property())));
        assert!(label.attributes().any(|attr| attr == &ControlAttribute::BoundingBox(Bounds::fill_all())));
    }

    #[test]
    fn can_create_container_with_components() {
        let container = Control::container()
            .with(vec![Control::label().with("Hello")]);

        assert!(container.control_type() == ControlType::Container);
        assert!(container.attributes().any(|attr| attr == &ControlAttribute::SubComponents(vec![Control::label().with("Hello")])));
    }

    #[test]
    fn can_find_all_subcontrollers() {
        let container = Control::container()
            .with(vec![
                Control::empty().with_controller("Test1"),
                Control::empty().with_controller("Test2"),
                Control::container().with(vec![
                    Control::empty().with_controller("Test3")
                ])
            ]);
        
        let subcontrollers = container.all_controllers();

        assert!(subcontrollers.len() == 3);
        assert!(subcontrollers.iter().any(|c| c == "Test1"));
        assert!(subcontrollers.iter().any(|c| c == "Test2"));
        assert!(subcontrollers.iter().any(|c| c == "Test3"));
    }

    #[test]
    fn will_only_report_subcontrollers_once() {
        let container = Control::container()
            .with(vec![
                Control::empty().with_controller("Test1"),
                Control::empty().with_controller("Test2"),
                Control::container().with(vec![
                    Control::empty().with_controller("Test1")
                ])
            ]);
        
        let subcontrollers = container.all_controllers();

        assert!(subcontrollers.len() == 2);
        assert!(subcontrollers.iter().any(|c| c == "Test1"));
        assert!(subcontrollers.iter().any(|c| c == "Test2"));
    }
}

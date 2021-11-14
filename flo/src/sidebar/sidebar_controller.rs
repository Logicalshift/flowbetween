use crate::model::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

///
/// Creates the user interface for the sidebar
///
fn sidebar_ui<Anim: 'static+EditableAnimation>(_model: &FloModel<Anim>) -> BindRef<Control> {
    use self::Position::*;

    let ui = bind(
        Control::container()
            .with(Bounds {
                x1: Start,
                y1: Start,
                x2: End,
                y2: End
            })
            .with(PointerBehaviour::ClickThrough)
            .with(vec![
                Control::button()
                    .with("Test")
                    .with(Bounds {
                        x1: Start,
                        y1: Start,
                        x2: End,
                        y2: Offset(30.0)
                    })
            ])
        );

    BindRef::from(ui)
}

///
/// Creates the sidebar controller
///
pub fn sidebar_controller<Anim: 'static+EditableAnimation>(model: &FloModel<Anim>) -> impl Controller {
    let model       = model.clone();
    let resources   = ControllerResources::new();
    let ui          = sidebar_ui(&model);

    ImmediateController::new(resources, ui, 
        move |_events, _actions, _resources| {
            // Start by taking the model from the main controller
            let model = model.clone();

            async move {

            }
        })
}

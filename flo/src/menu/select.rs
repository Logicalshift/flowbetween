use super::controls;
use super::super::model::*;
use super::super::standard_tools::*;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use desync::*;
use futures::*;
use futures::executor;
use futures::executor::Spawn;

use std::sync::*;
use std::collections::HashSet;

///
/// The menu controller for the selection tool
/// 
pub struct SelectMenuController<Anim: Animation> {
    /// Currently selected elements
    selected: BindRef<Arc<HashSet<ElementId>>>,

    /// The animation editing stream where this will send updates
    edit: Desync<Spawn<Box<dyn Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>+Send>>>,

    /// The timeline model for the animation
    timeline: TimelineModel<Anim>,

    // The UI for this control
    ui: BindRef<Control>
}

impl<Anim: 'static+EditableAnimation+Animation> SelectMenuController<Anim> {
    ///
    /// Creates a new select menu controller
    /// 
    pub fn new(flo_model: &FloModel<Anim>, tool_model: &SelectToolModel) -> SelectMenuController<Anim> {
        let ui          = Self::ui(tool_model);
        let edit        = Desync::new(executor::spawn(flo_model.edit()));
        let selected    = flo_model.selection().selected_element.clone();
        let timeline    = flo_model.timeline().clone();

        SelectMenuController {
            ui:         ui,
            edit:       edit,
            selected:   selected,
            timeline:   timeline
        }
    }

    ///
    /// Creates the UI for the select menu controller
    /// 
    fn ui(tool_model: &SelectToolModel) -> BindRef<Control> {
        let anything_selected   = tool_model.anything_selected.clone();
        let num_selected        = tool_model.num_elements_selected.clone();

        let ui              = 
            computed(move || {
                // Number of things selected
                let num_selected = num_selected.get();
                let num_selected = match num_selected {
                    0 => String::from("Nothing"),
                    1 => String::from("1 item"),
                    _ => format!("{} items", num_selected)
                };

                // Extra controls
                let selection_controls = if anything_selected.get() {
                    vec![
                        controls::divider(),

                        Control::label()
                            .with("Order:")
                            .with(TextAlign::Right)
                            .with(Font::Size(13.0))
                            .with(Bounds::next_horiz(48.0)),
                        Control::empty()
                            .with(Bounds::next_horiz(4.0)),
                        Control::container()
                            .with(Hint::Class("button-group".to_string()))
                            .with(ControlAttribute::Padding((0,2), (0,2)))
                            .with(Font::Size(9.0))
                            .with(Bounds::next_horiz(80.0))
                            .with(vec![
                                Control::button()
                                    .with(vec![Control::label().with("^^").with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "MoveToFront"))
                                    .with(Bounds::next_horiz(20.0)),
                                Control::button()
                                    .with(vec![Control::label().with("^").with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "MoveForwards"))
                                    .with(Bounds::next_horiz(20.0)),
                                Control::button()
                                    .with(vec![Control::label().with("v").with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "MoveBackwards"))
                                    .with(Bounds::next_horiz(20.0)),
                                Control::button()
                                    .with(vec![Control::label().with("vv").with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "MoveToBack"))
                                    .with(Bounds::next_horiz(20.0))
                            ])
                    ]
                } else {
                    vec![]
                };

                // Build the control
                Control::container()
                    .with(Bounds::fill_all())
                    .with(ControlAttribute::Padding((0, 3), (0, 3)))
                    .with(vec![
                        controls::divider(),

                        Control::label()
                            .with("Select:")
                            .with(FontWeight::Light)
                            .with(TextAlign::Right)
                            .with(Font::Size(14.0))
                            .with(Bounds::next_horiz(48.0)),

                        Control::label()
                            .with(num_selected)
                            .with(FontWeight::Light)
                            .with(TextAlign::Left)
                            .with(Font::Size(12.0))
                            .with(Bounds::next_horiz(56.0)),
                    ]
                    .into_iter()
                    .chain(selection_controls)
                    .collect::<Vec<_>>())
            });

        BindRef::from(ui)
    }
}

impl<Anim: 'static+EditableAnimation+Animation> Controller for SelectMenuController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        match action_id {
            "MoveToFront" | "MoveForwards" | "MoveBackwards" | "MoveToBack" => {
                let selection   = self.selected.get();
                let ordering    = match action_id {
                    "MoveToFront"   => ElementOrdering::ToTop,
                    "MoveForwards"  => ElementOrdering::InFront,
                    "MoveBackwards" => ElementOrdering::Behind,
                    "MoveToBack"    => ElementOrdering::ToBottom,
                    _               => ElementOrdering::ToTop
                };

                self.edit.sync(move |animation| { 
                    animation.wait_send(selection.iter()
                        .map(|selected_element| AnimationEdit::Element(*selected_element, ElementEdit::Order(ordering)))
                        .collect()).ok();
                    });
                self.timeline.invalidate_canvas();
            },

            _ => { }
        }
    }
}

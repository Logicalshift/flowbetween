use super::controls;
use super::super::model::*;
use super::super::standard_tools::*;

use flo_ui::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;

use ::desync::*;

use std::sync::*;
use std::collections::HashSet;

///
/// The menu controller for the selection tool
///
pub struct SelectMenuController<Anim: Animation> {
    /// Currently selected elements
    selected: BindRef<Arc<HashSet<ElementId>>>,

    /// Currently selected elements, in order
    selection_in_order: BindRef<Arc<Vec<ElementId>>>,

    /// The images for the selection menu
    images: Arc<ResourceManager<Image>>,

    /// The animation editing stream where this will send updates
    edit: Desync<Publisher<Arc<Vec<AnimationEdit>>>>,

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
        let images              = Self::images();
        let ui                  = Self::ui(&images, tool_model);
        let edit                = Desync::new(flo_model.edit());
        let selected            = flo_model.selection().selected_elements.clone();
        let selection_in_order  = flo_model.selection().selection_in_order.clone();
        let timeline            = flo_model.timeline().clone();

        SelectMenuController {
            ui:                 ui,
            edit:               edit,
            images:             Arc::new(images),
            selected:           selected,
            selection_in_order: selection_in_order,
            timeline:           timeline
        }
    }

    ///
    /// Creates the images for this controller
    ///
    fn images() -> ResourceManager<Image> {
        let images          = ResourceManager::new();

        let order_to_back   = images.register(svg_static(include_bytes!("../../svg/selection_controls/to_back.svg")));
        let order_behind    = images.register(svg_static(include_bytes!("../../svg/selection_controls/behind.svg")));
        let order_forward   = images.register(svg_static(include_bytes!("../../svg/selection_controls/forward.svg")));
        let order_to_front  = images.register(svg_static(include_bytes!("../../svg/selection_controls/to_front.svg")));

        images.assign_name(&order_to_back, "OrderToBack");
        images.assign_name(&order_behind, "OrderBehind");
        images.assign_name(&order_forward, "OrderForward");
        images.assign_name(&order_to_front, "OrderToFront");

        images
    }

    ///
    /// Creates the UI for the select menu controller
    ///
    fn ui(images: &ResourceManager<Image>, tool_model: &SelectToolModel) -> BindRef<Control> {
        let order_to_back       = images.get_named_resource("OrderToBack");
        let order_behind        = images.get_named_resource("OrderBehind");
        let order_forward       = images.get_named_resource("OrderForward");
        let order_to_front      = images.get_named_resource("OrderToFront");

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

                // Extra controls to display when there's a selection to edit
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
                            .with(Bounds::next_horiz(22.0*2.0 + 28.0*2.0))
                            .with(vec![
                                Control::button()
                                    .with(vec![Control::empty().with(order_to_back.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "MoveToFront"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((6, 0), (0, 2))),
                                Control::button()
                                    .with(vec![Control::empty().with(order_behind.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "MoveForwards"))
                                    .with(Bounds::next_horiz(22.0))
                                    .with(ControlAttribute::Padding((0, 0), (0, 2))),
                                Control::button()
                                    .with(vec![Control::empty().with(order_forward.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "MoveBackwards"))
                                    .with(Bounds::next_horiz(22.0))
                                    .with(ControlAttribute::Padding((0, 0), (0, 2))),
                                Control::button()
                                    .with(vec![Control::empty().with(order_to_front.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "MoveToBack"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((0, 0), (6, 2)))
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

                        Control::empty()
                            .with(Bounds::next_horiz(4.0)),

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

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        match action_id {
            "MoveToFront" | "MoveForwards" | "MoveBackwards" | "MoveToBack" => {
                let selection                       = self.selection_in_order.get();
                let (ordering, apply_in_reverse)    = match action_id {
                    "MoveToFront"   => (ElementOrdering::ToTop, true),
                    "MoveForwards"  => (ElementOrdering::InFront, true),
                    "MoveBackwards" => (ElementOrdering::Behind, false),
                    "MoveToBack"    => (ElementOrdering::ToBottom, false),
                    _               => (ElementOrdering::ToTop, true)
                };

                // TODO: brush elements have styles implied by the order that the elements are in the keyframe, so we need a way of editing the brush styles of the elements after re-ordering them here

                if apply_in_reverse {
                    let _ = self.edit.future(move |animation| {
                        animation.publish(Arc::new(vec![
                                AnimationEdit::Element(selection.iter().rev().cloned().collect(), ElementEdit::Order(ordering))
                            ]))
                        });
                } else {
                    let _ = self.edit.future(move |animation| {
                        animation.publish(Arc::new(vec![
                                AnimationEdit::Element(selection.iter().cloned().collect(), ElementEdit::Order(ordering))
                            ]))
                        });
                }
                self.edit.sync(|_| { });
                self.timeline.invalidate_canvas();
            },

            _ => { }
        }
    }
}

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

        let align_left      = images.register(svg_static(include_bytes!("../../svg/selection_controls/align_left.svg")));
        let align_center    = images.register(svg_static(include_bytes!("../../svg/selection_controls/align_center.svg")));
        let align_right     = images.register(svg_static(include_bytes!("../../svg/selection_controls/align_right.svg")));
        let align_top       = images.register(svg_static(include_bytes!("../../svg/selection_controls/align_top.svg")));
        let align_middle    = images.register(svg_static(include_bytes!("../../svg/selection_controls/align_middle.svg")));
        let align_bottom    = images.register(svg_static(include_bytes!("../../svg/selection_controls/align_bottom.svg")));

        let flip_horiz      = images.register(svg_static(include_bytes!("../../svg/selection_controls/flip_horizontal.svg")));
        let flip_vert       = images.register(svg_static(include_bytes!("../../svg/selection_controls/flip_vertical.svg")));

        let group           = images.register(svg_static(include_bytes!("../../svg/selection_controls/group.svg")));
        let ungroup         = images.register(svg_static(include_bytes!("../../svg/selection_controls/ungroup.svg")));
        let path_add        = images.register(svg_static(include_bytes!("../../svg/selection_controls/add.svg")));
        let path_subtract   = images.register(svg_static(include_bytes!("../../svg/selection_controls/subtract.svg")));
        let path_intersect  = images.register(svg_static(include_bytes!("../../svg/selection_controls/intersect.svg")));

        images.assign_name(&order_to_back, "OrderToBack");
        images.assign_name(&order_behind, "OrderBehind");
        images.assign_name(&order_forward, "OrderForward");
        images.assign_name(&order_to_front, "OrderToFront");

        images.assign_name(&align_left, "AlignLeft");
        images.assign_name(&align_center, "AlignCenter");
        images.assign_name(&align_right, "AlignRight");
        images.assign_name(&align_top, "AlignTop");
        images.assign_name(&align_middle, "AlignMiddle");
        images.assign_name(&align_bottom, "AlignBottom");

        images.assign_name(&flip_horiz, "FlipHorizontal");
        images.assign_name(&flip_vert, "FlipVertical");

        images.assign_name(&group, "Group");
        images.assign_name(&ungroup, "Ungroup");
        images.assign_name(&path_add, "PathAdd");
        images.assign_name(&path_subtract, "PathSubtract");
        images.assign_name(&path_intersect, "PathIntersect");

        images
    }

    ///
    /// Creates the UI for the select menu controller
    ///
    fn ui(images: &ResourceManager<Image>, tool_model: &SelectToolModel) -> BindRef<Control> {
        // Fetch the images
        let order_to_back       = images.get_named_resource("OrderToBack");
        let order_behind        = images.get_named_resource("OrderBehind");
        let order_forward       = images.get_named_resource("OrderForward");
        let order_to_front      = images.get_named_resource("OrderToFront");

        let align_left          = images.get_named_resource("AlignLeft");
        let align_center        = images.get_named_resource("AlignCenter");
        let align_right         = images.get_named_resource("AlignRight");
        let align_top           = images.get_named_resource("AlignTop");
        let align_middle        = images.get_named_resource("AlignMiddle");
        let align_bottom        = images.get_named_resource("AlignBottom");
        
        let flip_horiz          = images.get_named_resource("FlipHorizontal");
        let flip_vert           = images.get_named_resource("FlipVertical");

        let group               = images.get_named_resource("Group");
        let ungroup             = images.get_named_resource("Ungroup");
        let path_add            = images.get_named_resource("PathAdd");
        let path_subtract       = images.get_named_resource("PathSubtract");
        let path_intersect      = images.get_named_resource("PathIntersect");

        // Parts of the model
        let anything_selected   = tool_model.anything_selected.clone();
        let num_selected        = tool_model.num_elements_selected.clone();

        let ui              =
            computed(move || {
                // Number of things selected
                let num_selected        = num_selected.get();
                let num_selected_text   = match num_selected {
                    0 => String::from("Nothing"),
                    1 => String::from("1 item"),
                    _ => format!("{} items", num_selected)
                };

                let anything_selected   = anything_selected.get();
                let multi_select        = num_selected > 1;

                // Pick the control sets based on the selection
                let order_controls = if anything_selected { 
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

                let align_controls = if multi_select {
                    vec![
                        controls::divider(),

                        Control::label()
                            .with("Align:")
                            .with(TextAlign::Right)
                            .with(Font::Size(13.0))
                            .with(Bounds::next_horiz(48.0)),
                        Control::empty()
                            .with(Bounds::next_horiz(4.0)),
                        Control::container()
                            .with(Hint::Class("button-group".to_string()))
                            .with(ControlAttribute::Padding((0,2), (0,2)))
                            .with(Font::Size(9.0))
                            .with(Bounds::next_horiz(22.0*1.0 + 28.0*2.0))
                            .with(vec![
                                Control::button()
                                    .with(vec![Control::empty().with(align_left.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "AlignLeft"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((7, 1), (1, 3))),
                                Control::button()
                                    .with(vec![Control::empty().with(align_center.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "AlignCenter"))
                                    .with(Bounds::next_horiz(22.0))
                                    .with(ControlAttribute::Padding((1, 1), (1, 3))),
                                Control::button()
                                    .with(vec![Control::empty().with(align_right.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "AlignRight"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((1, 1), (7, 3)))
                            ]),
                        Control::empty()
                            .with(Bounds::next_horiz(4.0)),
                        Control::container()
                            .with(Hint::Class("button-group".to_string()))
                            .with(ControlAttribute::Padding((0,2), (0,2)))
                            .with(Font::Size(9.0))
                            .with(Bounds::next_horiz(22.0*1.0 + 28.0*2.0))
                            .with(vec![
                                Control::button()
                                    .with(vec![Control::empty().with(align_top.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "AlignTop"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((7, 1), (1, 3))),
                                Control::button()
                                    .with(vec![Control::empty().with(align_middle.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "AlignMiddle"))
                                    .with(Bounds::next_horiz(22.0))
                                    .with(ControlAttribute::Padding((1, 1), (1, 3))),
                                Control::button()
                                    .with(vec![Control::empty().with(align_bottom.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "AlignBottom"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((1, 1), (7, 3)))
                            ]),
                    ]
                } else {
                    vec![]
                };

                let flip_controls = if anything_selected {
                    vec![
                        controls::divider(),

                        Control::empty()
                            .with(Bounds::next_horiz(4.0)),
                        Control::container()
                            .with(Hint::Class("button-group".to_string()))
                            .with(ControlAttribute::Padding((0,2), (0,2)))
                            .with(Font::Size(9.0))
                            .with(Bounds::next_horiz(28.0*2.0))
                            .with(vec![
                                Control::button()
                                    .with(vec![Control::empty().with(flip_horiz.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "FlipHorizontal"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((7, 1), (1, 3))),
                                Control::button()
                                    .with(vec![Control::empty().with(flip_vert.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "FlipVertical"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((1, 1), (7, 3)))
                            ])
                    ]
                } else {
                    vec![]
                };

                let group_controls = if multi_select {
                    vec![
                        controls::divider(),

                        Control::label()
                            .with("Group:")
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
                                    .with(vec![Control::empty().with(group.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "Group"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((6, 0), (0, 2))),
                                Control::button()
                                    .with(vec![Control::empty().with(path_add.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "PathAdd"))
                                    .with(Bounds::next_horiz(22.0))
                                    .with(ControlAttribute::Padding((0, 0), (0, 2))),
                                Control::button()
                                    .with(vec![Control::empty().with(path_subtract.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "PathSubtract"))
                                    .with(Bounds::next_horiz(22.0))
                                    .with(ControlAttribute::Padding((0, 0), (0, 2))),
                                Control::button()
                                    .with(vec![Control::empty().with(path_intersect.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                                    .with(Font::Size(10.0))
                                    .with((ActionTrigger::Click, "PathIntersect"))
                                    .with(Bounds::next_horiz(28.0))
                                    .with(ControlAttribute::Padding((0, 0), (6, 2)))
                            ])
                    ]
                } else {
                    vec![]
                };

                // Extra controls to display when there's a selection to edit
                let selection_controls = order_controls.into_iter()
                    .chain(align_controls)
                    .chain(flip_controls)
                    .chain(group_controls);

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
                            .with(num_selected_text)
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
            // Ordering
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

            // Alignment
            "AlignLeft" | "AlignCenter" | "AlignRight" |
            "AlignTop" | "AlignMiddle" | "AlignBottom" => {
                // Convert to an alignment
                let selection       = self.selection_in_order.get();
                let element_align   = match action_id {
                    "AlignLeft"     => ElementAlign::Left,
                    "AlignCenter"   => ElementAlign::Center,
                    "AlignRight"    => ElementAlign::Right,

                    "AlignTop"      => ElementAlign::Top,
                    "AlignMiddle"   => ElementAlign::Middle,
                    "AlignBottom"   => ElementAlign::Bottom,

                    _               => ElementAlign::Center
                };

                // Send the edit
                let _ = self.edit.future(move |animation| {
                    animation.publish(Arc::new(vec![AnimationEdit::Element(selection.iter().cloned().collect(), 
                        ElementEdit::Transform(vec![ElementTransform::Align(element_align)]))]))
                });
                self.edit.sync(|_| { });
                self.timeline.invalidate_canvas();
            }

            // Flipping about an axis
            "FlipHorizontal" => {
                let selection   = self.selection_in_order.get();

                let _           = self.edit.future(move |animation| {
                    animation.publish(Arc::new(vec![AnimationEdit::Element(selection.iter().cloned().collect(), 
                        ElementEdit::Transform(vec![ElementTransform::FlipHorizontal]))]))
                });
                self.edit.sync(|_| { });
                self.timeline.invalidate_canvas();
            }

            "FlipVertical" => {
                let selection   = self.selection_in_order.get();

                let _           = self.edit.future(move |animation| {
                    animation.publish(Arc::new(vec![AnimationEdit::Element(selection.iter().cloned().collect(), 
                        ElementEdit::Transform(vec![ElementTransform::FlipVertical]))]))
                });
                self.edit.sync(|_| { });
                self.timeline.invalidate_canvas();
            }

            _ => { }
        }
    }
}

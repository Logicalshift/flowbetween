use super::property_action::*;
use super::super::gtk_action::*;

use flo_ui::*;

use std::sync::*;

pub type PropertyWidgetAction = PropertyAction<GtkWidgetAction>;

///
/// Trait implemented by things that can be converted to GTK widget actions
///
pub trait ToGtkActions {
    ///
    /// Converts this itme to a set of GtkWidgetActions required to render it to a GTK widget
    ///
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction>;
}

///
/// Determines the GTK control to use for a button control
///
/// We need to use a toggle button for buttons with a selected attribute, or a normal button for everything else
///
fn button_type_for_control(control: &Control) -> GtkWidgetType {
    use self::ControlAttribute::*;

    // This works because we don't update attributes except via the viewmodel right now (so if the control hasn't got a selected
    // attribute it will never have one in the future). If at some point, the selected attribute can be added to a control, we may
    // end up with a situation where Button is chosen incorrectly.
    if control.attributes().any(|attribute| match attribute { &StateAttr(State::Selected(_)) => true, _ => false }) {
        GtkWidgetType::ToggleButton
    } else {
        GtkWidgetType::Button
    }
}

///
/// Determines the GTK control to use for a canvas control
///
/// Normally we want to use drawing areas, but sometimes canvases can have subcontrols, and in that case we need to use a layout
/// so we can draw a canvas and have other controls on top of it
///
fn canvas_type_for_control(control: &Control) -> GtkWidgetType {
    let fast_drawing = control.attributes().any(|attr| attr == &ControlAttribute::HintAttr(Hint::FastDrawing));

    if fast_drawing {
        GtkWidgetType::CanvasRender
    } else if let Some(subcomponents) = control.subcomponents() {
        if subcomponents.len() > 0 {
            GtkWidgetType::CanvasLayout
        } else {
            GtkWidgetType::CanvasDrawingArea
        }
    } else {
        GtkWidgetType::CanvasDrawingArea
    }
}

///
/// Determines if a control needs to use an overlay or not
///
fn needs_overlay(control: &Control) -> bool {
    use self::ControlAttribute::*;

    control.attributes().any(|attribute| {
        match attribute {
            // Images are drawn under any child controls, so an overlay should be used
            &AppearanceAttr(Appearance::Image(_))   => true,

            // Other controls do not need an event box
            _                                       => false
        }
    })
}

///
/// Determines if a control needs an event box or not
///
fn needs_event_box(control: &Control) -> bool {
    use self::ControlAttribute::*;

    control.attributes().any(|attribute| {
        match attribute {
            // Controls with a background colour need an event box
            &AppearanceAttr(Appearance::Background(_))  => true,

            // Controls with a z-index need an event box
            &ZIndex(_)                                  => true,

            // Controls with painting need to turn off event compression
            &Action(ActionTrigger::Paint(_), _)         => true,

            // Controls that can be clicked or dragged need an event box for their target
            &Action(ActionTrigger::Click, _)            => true,
            &Action(ActionTrigger::Drag, _)             => true,

            // Other controls do not need an event box
            _                                           => false
        }
    })
}

///
/// Creates the actions required to instantiate a control - without its subcomponents
///
impl ToGtkActions for Control {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::ControlType::*;
        use self::GtkWidgetAction::New;

        // Convert the control type into the command to create the appropriate Gtk widget
        let create_new = match self.control_type() {
            Empty               => New(GtkWidgetType::Generic),
            Container           => if needs_overlay(self) { New(GtkWidgetType::Overlay) } else { New(GtkWidgetType::Fixed) },
            CroppingContainer   => New(GtkWidgetType::Layout),
            ScrollingContainer  => New(GtkWidgetType::ScrollArea),
            Popup               => New(GtkWidgetType::Popover),
            Button              => New(button_type_for_control(self)),
            Label               => New(GtkWidgetType::Label),
            Canvas              => New(canvas_type_for_control(self)),
            Slider              => New(GtkWidgetType::Scale),
            Rotor               => New(GtkWidgetType::Rotor),
            TextBox             => New(GtkWidgetType::TextBox),
            CheckBox            => New(GtkWidgetType::CheckBox)
        };

        // The widget class allows the style sheet to specifically target Flo widgets
        let widget_class = match self.control_type() {
            Empty               => "flo-empty",
            Container           => "flo-container",
            CroppingContainer   => "flo-cropping-container",
            ScrollingContainer  => "flo-scrolling-container",
            Popup               => "flo-popup",
            Button              => "flo-button",
            Label               => "flo-label",
            Canvas              => "flo-canvas",
            Slider              => "flo-slider",
            Rotor               => "flo-rotor",
            TextBox             => "flo-textbox",
            CheckBox            => "flo-checkbox"
        };

        // Build into the 'create control' action
        let mut create_control = vec![
            create_new.into(),
        ];

        // Some controls need an event box (eg, to show a custom background or to allow for z-ordering)
        if needs_event_box(self) {
            create_control.push(GtkWidgetAction::IntoEventBox.into());
        }

        // Controls have their own class for styling and are displayed by default
        create_control.extend(vec![
            GtkWidgetAction::Content(WidgetContent::AddClass(widget_class.to_string())).into(),
            GtkWidgetAction::Show.into()
        ]);

        // Generate the actions for all of the attributes
        for attribute in self.attributes() {
            let create_attribute = attribute.to_gtk_actions();
            create_control.extend(create_attribute.into_iter());
        }

        create_control
    }
}

impl ToGtkActions for ControlAttribute {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::ControlAttribute::*;

        match self {
            &BoundingBox(ref bounds)                => bounds.to_gtk_actions(),
            &ZIndex(zindex)                         => vec![ WidgetLayout::ZIndex(zindex).into() ].into_actions(),
            &Padding((left, top), (right, bottom))  => vec![ WidgetLayout::Padding((left, top), (right, bottom)).into() ].into_actions(),

            &Text(ref text)                         => vec![ PropertyAction::from_property(text.clone(), |text| vec![ WidgetContent::SetText(text.to_string()).into() ]) ],

            &FontAttr(ref font)                     => font.to_gtk_actions(),
            &StateAttr(ref state)                   => state.to_gtk_actions(),
            &PopupAttr(ref popup)                   => popup.to_gtk_actions(),
            &AppearanceAttr(ref appearance)         => appearance.to_gtk_actions(),
            &ScrollAttr(ref scroll)                 => scroll.to_gtk_actions(),
            &HintAttr(ref hint)                     => hint.to_gtk_actions(),

            &Id(ref id)                             => vec![ WidgetContent::AddClass(id.clone()).into() ].into_actions(),
            &Action(ref _trigger, ref _action_name) => vec![],

            // TODO: canvas drawing instructions are needed for canvases that have been 'seen' before, but for entirely new canvases
            // there will be an initial update that will duplicate these actions (fortunately starting with a clear so it's not user
            // visible)
            &Canvas(ref canvas)                     => vec![ WidgetContent::Draw(canvas.get_drawing()).into() ].into_actions(),

            // The GTK layout doesn't need to know the controller
            &Controller(ref controller_name)       => vec![ GtkWidgetAction::Content(WidgetContent::AddClass(format!("c-{}", controller_name))) ].into_actions(),

            // Subcomponents are added elsewhere: we don't assign them here
            &SubComponents(ref _components)         => vec![]
        }
    }
}

impl ToGtkActions for Hint {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        match self {
            Hint::FastDrawing       => vec![],
            Hint::Class(class_name) => vec![ GtkWidgetAction::Content(WidgetContent::AddClass(class_name.clone())) ].into_actions()
        }
    }
}

impl ToGtkActions for Bounds {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        // We always start by setting the bounding box to the active value
        let mut result = vec![ PropertyAction::Unbound(WidgetLayout::BoundingBox(self.clone()).into()) ];

        // The bounding box may have floating properties in x1 and y1 (we don't support x2 and y2 yet). These get property bindings that trigger relayouts
        let floating_point = Arc::new(Mutex::new((0.0, 0.0)));

        // If the x1 position is floating, then generate a property binding to update it
        if let &Position::Floating(ref property, ref _offset) = &self.x1 {
            let floating_point  = Arc::clone(&floating_point);

            result.push(PropertyAction::from_property(property.clone(), move |x1_floating| {
                // Update the floating point
                let mut floating_point = floating_point.lock().unwrap();

                if let Some(x1_floating) = x1_floating.to_f64() {
                    floating_point.0 = x1_floating;
                }

                vec![ WidgetLayout::Floating(floating_point.0, floating_point.1).into() ]
            }).into());
        }

        // If the y1 position is floating, then generate a property binding to update it
        if let &Position::Floating(ref property, ref _offset) = &self.y1 {
            let floating_point  = Arc::clone(&floating_point);

            result.push(PropertyAction::from_property(property.clone(), move |y1_floating| {
                // Update the floating point
                let mut floating_point = floating_point.lock().unwrap();

                if let Some(y1_floating) = y1_floating.to_f64() {
                    floating_point.0 = y1_floating;
                }

                vec![ WidgetLayout::Floating(floating_point.0, floating_point.1).into() ]
            }).into());
        }

        result
    }
}

impl ToGtkActions for State {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::State::*;

        match self {
            Selected(ref selected)      => vec![ PropertyAction::from_property(selected.clone(), |value| vec![ WidgetState::SetSelected(value.to_bool().unwrap_or(false)).into() ]) ],
            Badged(ref badged)          => vec![ PropertyAction::from_property(badged.clone(), |value| vec![ WidgetState::SetBadged(value.to_bool().unwrap_or(false)).into() ]) ],
            Enabled(ref enabled)        => vec![ PropertyAction::from_property(enabled.clone(), |value| vec![ WidgetState::SetEnabled(value.to_bool().unwrap_or(true)).into() ]) ],
            Range((ref min, ref max))   => vec![
                PropertyAction::from_property(min.clone(), |min| vec![ WidgetState::SetRangeMin(min.to_f64().unwrap_or(0.0)).into() ]),
                PropertyAction::from_property(max.clone(), |max| vec![ WidgetState::SetRangeMax(max.to_f64().unwrap_or(0.0)).into() ])
            ],
            FocusPriority(ref priority) => vec![], /* TODO */

            Value(ref value)            => vec![ PropertyAction::from_property(value.clone(), |value| {
                match value {
                    PropertyValue::Bool(val)    => vec![WidgetState::SetValueBool(val).into()],
                    PropertyValue::Float(val)   => vec![WidgetState::SetValueFloat(val).into()],
                    PropertyValue::Int(val)     => vec![WidgetState::SetValueInt(val as i64).into()],
                    PropertyValue::String(val)  => vec![WidgetState::SetValueText(val).into()],
                    _ => vec![WidgetState::SetValueFloat(0.0).into()]
                }
            }) ],
        }
    }
}

impl ToGtkActions for Popup {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::Popup::*;

        match self {
            &IsOpen(ref is_open)        => vec![ PropertyAction::from_property(is_open.clone(), |is_open| vec![ WidgetPopup::SetOpen(is_open.to_bool().unwrap_or(false)).into() ])],
            &Direction(direction)       => vec![ WidgetPopup::SetDirection(direction).into() ].into_actions(),
            &Size(width, height)        => vec![ WidgetPopup::SetSize(width, height).into() ].into_actions(),
            &Offset(distance)           => vec![ WidgetPopup::SetOffset(distance).into() ].into_actions()
        }
    }
}

impl ToGtkActions for Font {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        vec![ self.clone().into() ].into_actions()
    }
}

impl ToGtkActions for Appearance {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        vec![ self.clone().into() ].into_actions()
    }
}

impl ToGtkActions for Scroll {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        vec![ self.clone().into() ].into_actions()
    }
}

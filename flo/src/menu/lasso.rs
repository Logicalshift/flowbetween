use super::controls;

use flo_ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;

///
/// The mode to use when selecting a region of the canvas using the lasso tool
///
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LassoMode {
    /// Select a new region (or drag the existing one)
    Select,

    /// Add to the existing region
    Add,

    /// Subtract from the existing region
    Subtract,

    /// Intersect with the existing region
    Intersect
}

///
/// The menu controller for the lasso tool
///
pub struct LassoMenuController {
    /// How the selection is updated when using the lasso
    lasso_mode: Binding<LassoMode>,

    images:     Arc<ResourceManager<Image>>,
    ui:         BindRef<Control>,
    view_model: Arc<DynamicViewModel>,
}

impl LassoMenuController {
    ///
    /// Creates a new lasso menu controller
    ///
    pub fn new(lasso_mode: &Binding<LassoMode>, selected_path: &Binding<Option<Arc<Path>>>) -> LassoMenuController {
        let images          = Arc::new(Self::images());
        let lasso_mode      = lasso_mode.clone();
        let selected_path   = selected_path.clone();

        let view_model      = Self::view_model(&lasso_mode, &selected_path);
        let ui              = Self::ui(&images, &lasso_mode);

        LassoMenuController {
            lasso_mode: lasso_mode,
            images:     images,
            view_model: view_model,
            ui:         ui
        }
    }

    ///
    /// Loads the images for this menu controller
    ///
    fn images() -> ResourceManager<Image> {
        let images = ResourceManager::new();

        let select                  = images.register(svg_static(include_bytes!("../../svg/selection_controls/lasso_cursor.svg")));
        let add                     = images.register(svg_static(include_bytes!("../../svg/selection_controls/add.svg")));
        let subtract                = images.register(svg_static(include_bytes!("../../svg/selection_controls/subtract.svg")));
        let intersect               = images.register(svg_static(include_bytes!("../../svg/selection_controls/intersect.svg")));

        images.assign_name(&select,     "Select");
        images.assign_name(&add,        "Add");
        images.assign_name(&subtract,   "Subtract");
        images.assign_name(&intersect,  "Intersect");

        images
    }

    ///
    /// Creates the viewmodel for this controller
    ///
    fn view_model(lasso_mode: &Binding<LassoMode>, selected_path: &Binding<Option<Arc<Path>>>) -> Arc<DynamicViewModel> {
        let view_model  = Arc::new(DynamicViewModel::new());

        let mode = lasso_mode.clone(); let path = selected_path.clone(); view_model.set_computed("ModeSelect",      move || PropertyValue::Bool(path.get().is_none() || mode.get() == LassoMode::Select));
        let mode = lasso_mode.clone(); let path = selected_path.clone(); view_model.set_computed("ModeAdd",         move || PropertyValue::Bool(path.get().is_some() && mode.get() == LassoMode::Add));
        let mode = lasso_mode.clone(); let path = selected_path.clone(); view_model.set_computed("ModeSubtract",    move || PropertyValue::Bool(path.get().is_some() && mode.get() == LassoMode::Subtract));
        let mode = lasso_mode.clone(); let path = selected_path.clone(); view_model.set_computed("ModeIntersect",   move || PropertyValue::Bool(path.get().is_some() && mode.get() == LassoMode::Intersect));


        let path = selected_path.clone(); view_model.set_computed("EnableAdd",         move || PropertyValue::Bool(path.get().is_some()));
        let path = selected_path.clone(); view_model.set_computed("EnableSubtract",    move || PropertyValue::Bool(path.get().is_some()));
        let path = selected_path.clone(); view_model.set_computed("EnableIntersect",   move || PropertyValue::Bool(path.get().is_some()));

        view_model
    }

    ///
    /// Creates the UI binding for the lasso controler
    ///
    fn ui(images: &Arc<ResourceManager<Image>>, lasso_mode: &Binding<LassoMode>) -> BindRef<Control> {
        // Copy the resources
        let images      = images.clone();
        let lasso_mode  = lasso_mode.clone();

        let select      = images.get_named_resource("Select");
        let add         = images.get_named_resource("Add");
        let subtract    = images.get_named_resource("Subtract");
        let intersect   = images.get_named_resource("Intersect");

        // Build the UI
        BindRef::from(computed(move || {
            let mode_control = Control::container()
                .with(Hint::Class("button-group".to_string()))
                .with(ControlAttribute::Padding((0,2), (0,2)))
                .with(Font::Size(9.0))
                .with(Bounds::next_horiz(28.0*2.0))
                .with(vec![
                    Control::button()
                        .with(vec![Control::empty().with(select.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                        .with(Font::Size(10.0))
                        .with((ActionTrigger::Click, "ModeSelect"))
                        .with(Hover::Tooltip("Select new region".to_string()))
                        .with(State::Selected(Property::bound("ModeSelect")))
                        .with(Bounds::next_horiz(28.0))
                        .with(ControlAttribute::Padding((7, 1), (1, 3))),
                    Control::button()
                        .with(vec![Control::empty().with(add.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                        .with(Font::Size(10.0))
                        .with((ActionTrigger::Click, "ModeAdd"))
                        .with(Hover::Tooltip("Add to region".to_string()))
                        .with(State::Selected(Property::bound("ModeAdd")))
                        .with(State::Enabled(Property::bound("EnableAdd")))
                        .with(Bounds::next_horiz(28.0))
                        .with(ControlAttribute::Padding((1, 1), (1, 3))),
                    Control::button()
                        .with(vec![Control::empty().with(subtract.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                        .with(Font::Size(10.0))
                        .with((ActionTrigger::Click, "ModeSubtract"))
                        .with(Hover::Tooltip("Remove from region".to_string()))
                        .with(State::Selected(Property::bound("ModeSubtract")))
                        .with(State::Enabled(Property::bound("EnableSubtract")))
                        .with(Bounds::next_horiz(28.0))
                        .with(ControlAttribute::Padding((1, 1), (1, 3))),
                    Control::button()
                        .with(vec![Control::empty().with(intersect.clone()).with(TextAlign::Center).with(Bounds::fill_all())])
                        .with(Font::Size(10.0))
                        .with((ActionTrigger::Click, "ModeIntersect"))
                        .with(Hover::Tooltip("Intersect with region".to_string()))
                        .with(State::Selected(Property::bound("ModeIntersect")))
                        .with(State::Enabled(Property::bound("EnableIntersect")))
                        .with(Bounds::next_horiz(28.0))
                        .with(ControlAttribute::Padding((1, 1), (7, 3)))
                ]);

            Control::container()
                .with(Bounds::fill_all())
                .with(ControlAttribute::Padding((0, 3), (0, 3)))
                .with(vec![
                    controls::divider(),

                    Control::label()
                        .with("Lasso:")
                        .with(FontWeight::Light)
                        .with(TextAlign::Right)
                        .with(Font::Size(14.0))
                        .with(Bounds::next_horiz(48.0)),

                    Control::empty()
                        .with(Bounds::next_horiz(4.0)),

                    mode_control
                ])
            }))
    }
}

impl Controller for LassoMenuController {
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }

    fn get_viewmodel(&self) -> Option<Arc<dyn ViewModel>> {
        Some(self.view_model.clone())
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        match action_id {
            "ModeSelect"    => self.lasso_mode.set(LassoMode::Select),
            "ModeAdd"       => self.lasso_mode.set(LassoMode::Add),
            "ModeSubtract"  => self.lasso_mode.set(LassoMode::Subtract),
            "ModeIntersect" => self.lasso_mode.set(LassoMode::Intersect),

            _ => { }
        }
    }
}

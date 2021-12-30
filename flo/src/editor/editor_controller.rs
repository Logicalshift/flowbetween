use super::menu_controller::*;
use super::canvas_controller::*;
use super::toolbox_controller::*;
use super::timeline_controller::*;
use super::keybinding_controller::*;
use super::controlbar_controller::*;

use crate::model::*;
use crate::style::*;
use crate::sidebar::*;

use flo_ui::*;
use flo_ui_files::ui::*;
use flo_binding::*;
use flo_animation::*;

use std::sync::*;
use std::str::{FromStr};
use std::marker::{PhantomData};
use std::collections::{HashMap};

#[derive(Clone, PartialEq, Eq, Hash, AsRefStr, Display, EnumString)]
enum SubController {
    Canvas,
    Menu,
    ControlBar,
    Timeline,
    Toolbox,
    KeyBindings,
    Sidebar
}

///
/// The editor controller manages the editing of a single file
///
pub struct EditorController<Anim: FileAnimation> {
    /// Phantom data so we can have the animation type
    anim: PhantomData<Anim>,

    /// The sidebar model
    sidebar_model: SidebarModel,

    /// The images for the editor controller
    images: Arc<ResourceManager<Image>>,

    /// The main editor UI
    ui: BindRef<Control>,

    /// The subcontrollers for this editor
    subcontrollers: HashMap<SubController, Arc<dyn Controller>>
}

impl<Loader: 'static+FileAnimation> EditorController<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
    ///
    /// Creates a new editor controller from an animation
    ///
    pub fn new(animation: Loader::NewAnimation) -> EditorController<Loader> {
        let animation   = FloModel::new(animation);

        Self::from_model(animation)
    }

    ///
    /// Creates a new editor controller from a model
    ///
    pub fn from_model(animation: FloModel<Loader::NewAnimation>) -> EditorController<Loader> {
        // Load the image resources
        let images          = Arc::new(ResourceManager::new());

        images.register_named("sidebar_closed_active", svg_static(include_bytes!("../../svg/control_decals/sidebar_closed_active.svg")));
        images.register_named("sidebar_closed_inactive", svg_static(include_bytes!("../../svg/control_decals/sidebar_closed_inactive.svg")));
        images.register_named("sidebar_open", svg_static(include_bytes!("../../svg/control_decals/sidebar_open.svg")));

        // Create the subcontrollers
        let canvas          = Arc::new(CanvasController::new(&animation));
        let menu            = Arc::new(MenuController::new(&animation));
        let timeline        = Arc::new(TimelineController::new(&animation));
        let toolbox         = Arc::new(ToolboxController::new(&animation));
        let control_bar     = Arc::new(ControlBarController::new(&animation));
        let key_bindings    = Arc::new(KeyBindingController::new());
        let sidebar         = Arc::new(sidebar_controller(&animation));

        let ui              = Self::ui(&animation, &images);
        let mut subcontrollers: HashMap<SubController, Arc<dyn Controller>> = HashMap::new();

        subcontrollers.insert(SubController::Canvas,        canvas);
        subcontrollers.insert(SubController::Menu,          menu);
        subcontrollers.insert(SubController::Timeline,      timeline);
        subcontrollers.insert(SubController::Toolbox,       toolbox);
        subcontrollers.insert(SubController::ControlBar,    control_bar);
        subcontrollers.insert(SubController::KeyBindings,   key_bindings);
        subcontrollers.insert(SubController::Sidebar,       sidebar);

        EditorController {
            anim:           PhantomData,
            images:         images,
            ui:             ui,
            subcontrollers: subcontrollers,
            sidebar_model:  animation.sidebar().clone()
        }
    }

    ///
    /// Creates the menu bar control for this session
    ///
    fn menu_bar() -> Control {
        use self::Position::*;

        Control::container()
            .with(Bounds {
                x1: Start,
                y1: After,
                x2: End,
                y2: Offset(32.0)
            })
            .with_controller(&SubController::Menu.to_string())
    }

    ///
    /// Creates the timeline control
    ///
    pub fn timeline() -> Control {
        use self::Position::*;

        Control::container()
            .with(Bounds {
                x1: Start,
                y1: After,
                x2: End,
                y2: Offset(256.0)
            })
            .with_controller(&SubController::Timeline.to_string())
    }

    ///
    /// Creates the toolbar control
    ///
    pub fn toolbox() -> Control {
        use self::Position::*;

        Control::container()
            .with(Bounds {
                x1: Start,
                y1: After,
                x2: Offset(TOOL_CONTROL_SIZE),
                y2: End
            })
            .with_controller(&SubController::Toolbox.to_string())
    }

    ///
    /// Creates the canvas control
    ///
    pub fn canvas() -> Control {
        use self::Position::*;

        Control::container()
            .with(Bounds {
                x1: After,
                y1: Start,
                x2: Stretch(1.0),
                y2: End
            })
            .with(vec![
                Control::container()
                    .with(Bounds {
                        x1: Start,
                        y1: Start,
                        x2: End,
                        y2: End
                    })
                    .with(ControlAttribute::ZIndex(0))
                    .with_controller(&SubController::Canvas.to_string())
            ])
    }

    ///
    /// Creates the sidebar control
    ///
    pub fn sidebar(open_state: &BindRef<SidebarOpenState>, activation_state: &BindRef<SidebarActivationState>, images: &ResourceManager<Image>) -> Control {
        use self::Position::*;

        // The sidebar has an open state (which is controlled by the user) and an activation state (controller by the program)
        // Whether or not it is displayed as 'open' depends on both states
        let open_state          = open_state.get();
        let activation_state    = activation_state.get();

        if open_state.is_open(&activation_state) {
            // The sidebar should be displayed as open
            let open_image = images.get_named_resource("sidebar_open").unwrap();

            Control::container()
                .with(Bounds {
                    x1: Offset(0.0),
                    y1: Start,
                    x2: Offset(300.0),
                    y2: End
                })
                .with(Appearance::Background(SIDEBAR_BACKGROUND))
                .with((ActionTrigger::Click, "CloseSidebar"))
                .with(vec![
                    Control::empty().with(Bounds { x1: Start, y1: Start, x2: Offset(2.0), y2: End }),
                    Control::container().with(Bounds { x1: After, y1: Start, x2: Offset(16.0), y2: End }).with(PointerBehaviour::ClickThrough)
                        .with(vec![
                            Control::empty().with(Bounds { x1: Start, y1: Start, x2: End, y2: Stretch(0.5) }).with(PointerBehaviour::ClickThrough),
                            Control::empty().with(Bounds { x1: Start, y1: After, x2: End, y2: Offset(28.0) }).with(open_image).with(PointerBehaviour::ClickThrough),
                            Control::empty().with(Bounds { x1: Start, y1: After, x2: End, y2: Stretch(0.5) }).with(PointerBehaviour::ClickThrough)
                        ]),
                    Control::empty().with(Bounds { x1: After, y1: Start, x2: Offset(2.0), y2: End }),
                    Control::container().with(Bounds { x1: After, y1: Start, x2: End, y2: End }).with(PointerBehaviour::ClickThrough).with_controller(&SubController::Sidebar.to_string())
                ])
        } else {
            // The sidebar should be displayed as closed
            let closed_image = match activation_state {
                SidebarActivationState::Active      => images.get_named_resource("sidebar_closed_active").unwrap(),
                SidebarActivationState::Inactive    => images.get_named_resource("sidebar_closed_inactive").unwrap(),
            };

            Control::container()
                .with(Bounds {
                    x1: Offset(0.0),
                    y1: Start,
                    x2: Offset(32.0),
                    y2: End
                })
                .with((ActionTrigger::Click, "OpenSidebar"))
                .with(Appearance::Background(TOOLS_BACKGROUND))
                .with(vec![
                    Control::empty().with(Bounds { x1: Start, y1: Start, x2: End, y2: Stretch(0.5) }).with(PointerBehaviour::ClickThrough),
                    Control::empty().with(Bounds { x1: Start, y1: After, x2: End, y2: Offset(28.0) }).with(closed_image).with(PointerBehaviour::ClickThrough),
                    Control::empty().with(Bounds { x1: Start, y1: After, x2: End, y2: Stretch(0.5) }).with(PointerBehaviour::ClickThrough)
                ])
        }
    }

    ///
    /// Creates the control bar control
    ///
    pub fn control_bar() -> Control {
        Control::container()
            .with(Bounds::next_vert(28.0))
            .with(Appearance::Background(TIMESCALE_BACKGROUND))
            .with(Font::Size(12.0))
            .with(Font::Weight(FontWeight::Light))
            .with_controller(&SubController::ControlBar.to_string())
    }

    ///
    /// Creates the key bindings control
    ///
    pub fn keybindings() -> Control {
        Control::container()
            .with(Bounds::next_vert(0.0))
            .with_controller(&SubController::KeyBindings.to_string())
    }

    ///
    /// Creates the UI tree for this controller
    ///
    pub fn ui(model: &FloModel<Loader::NewAnimation>, images: &Arc<ResourceManager<Image>>) -> BindRef<Control> {
        use self::Position::*;

        let sidebar_open        = BindRef::from(model.sidebar().open_state.clone());
        let sidebar_activated   = model.sidebar().activation_state.clone();
        let images              = Arc::clone(images);

        let ui = computed(move || {
            // Basic controls
            let menu_bar            = Self::menu_bar();
            let timeline            = Self::timeline();
            let toolbar             = Self::toolbox();
            let canvas              = Self::canvas();
            let sidebar             = Self::sidebar(&sidebar_open, &sidebar_activated, &*images);
            let control_bar         = Self::control_bar();
            let keybindings         = Self::keybindings();

            // Assembly containing the tool panel, the canvas and the main timeline stacked on top of each other
            let canvas_and_timeline = Control::container()
                .with(vec![
                    Control::container()
                        .with((vec![toolbar, canvas],
                            Bounds { x1: Start, y1: Start, x2: End, y2: Stretch(1.0) })),
                    Control::empty()
                        .with(Bounds::next_vert(1.0))
                        .with(Appearance::Background(TIMESCALE_BORDER)),
                    control_bar,
                    Control::empty()
                        .with(Bounds::next_vert(1.0))
                        .with(Appearance::Background(TIMESCALE_LAYERS)),
                    timeline
                ]);

            // Put it all together
            Control::container()
                .with(Bounds::fill_all())
                .with(vec![
                    keybindings,
                    menu_bar,

                    Control::container()
                        .with(Bounds { x1: Start, y1: After, x2: End, y2: Stretch(1.0) })
                        .with(vec![
                            canvas_and_timeline
                                .with(Bounds { x1: Start, y1: Start, x2: Stretch(1.0), y2: End }),
                            sidebar
                        ])
                ])
            });

        BindRef::from(ui)
    }
}

impl<Loader: 'static+FileAnimation> Controller for EditorController<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
    fn ui(&self) -> BindRef<Control> {
        BindRef::new(&self.ui)
    }

    fn get_subcontroller(&self, id: &str) -> Option<Arc<dyn Controller>> {
        let decoded_id = SubController::from_str(id).ok()?;

        self.subcontrollers.get(&decoded_id)
            .map(|controller_ref| controller_ref.clone())
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }

    fn action(&self, action_id: &str, action_data: &ActionParameter) { 
        match (action_id, action_data) {
            ("OpenSidebar", _)      => { 
                // The sidebar is set to 'always open' if it's inactive, otherwise it is set to close when deactivated
                let open_state = match self.sidebar_model.activation_state.get() {
                    SidebarActivationState::Active      => SidebarOpenState::OpenWhenActive,
                    SidebarActivationState::Inactive    => SidebarOpenState::AlwaysOpen
                };

                self.sidebar_model.open_state.set(open_state); 
            }

            ("CloseSidebar", _)     => { 
                // The sidebar is set to 'always closed' if it's active, otherwise it's set to close when deactivated
                let open_state = match self.sidebar_model.activation_state.get() {
                    SidebarActivationState::Active      => SidebarOpenState::AlwaysClosed,
                    SidebarActivationState::Inactive    => SidebarOpenState::OpenWhenActive
                };

                self.sidebar_model.open_state.set(open_state); 
            }

            _                       => { /* Unknown action */ }
        }
    }
}

impl<Loader: 'static+FileAnimation> FileController for EditorController<Loader>
where Loader::NewAnimation: 'static+EditableAnimation {
    /// The model that this controller needs to be constructed
    type Model = FloSharedModel<Loader>;

    ///
    /// Creates this controller with the specified instance model
    ///
    fn open(model: FloModel<Loader::NewAnimation>) -> Self {
        Self::from_model(model)
    }
}

impl<Loader: 'static+FileAnimation> PartialEq for EditorController<Loader> {
    fn eq(&self, _rhs: &Self) -> bool {
        // These are never equal at the moment (this is needed for the binding library)
        false
    }
}

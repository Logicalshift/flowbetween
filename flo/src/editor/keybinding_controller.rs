use flo_ui::*;
use flo_binding::*;

///
/// The keybinding controller adds no UI but defines the 'standard' keybindings for FlowBetween
///
/// Note that some keybindings are more dynamic (eg, the toolbox defines its own set of keybindings, and individual tools
/// may add their own shortcuts)
///
pub struct KeyBindingController {
    ui:                 BindRef<Control>,
}

impl KeyBindingController {
    ///
    /// Creates a new menu controller
    ///
    pub fn new() -> KeyBindingController {
        // Keybinding controller is just a UI containing an empty control with a bunch of keybindings
        let ui = Self::create_ui();
        KeyBindingController {
            ui: BindRef::from(ui),
        }
    }

    ///
    /// Creates the UI binding for this controller
    ///
    fn create_ui() -> Binding<Control> {
        use self::KeyPress::*;

        bind(Control::empty()
            .with((KeyBinding::ctrl(KeyNumpadAdd), Command::with_id("layer_add")))

            .with((KeyBinding::ctrl(KeyZ), Command::with_id("undo")))
            .with((KeyBinding::ctrl_shift(KeyZ), Command::with_id("redo")))
        )
    }
}

impl Controller for KeyBindingController  {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }
}

use ui::*;
use binding::*;

///
/// Controller that makes it possible to pick a colour
/// 
pub struct ColorPickerController {
    ui: BindRef<Control>
}

impl ColorPickerController {
    ///
    /// Creates a new color picker controller
    /// 
    pub fn new() -> ColorPickerController {
        let ui = Self::create_ui();

        ColorPickerController {
            ui: ui
        }
    }

    fn create_ui() -> BindRef<Control> {
        BindRef::from(computed(move || {
            Control::label()
                .with("Hello, colour controller")
                .with(Bounds::fill_all())
        }))
    }
}

impl Controller for ColorPickerController {
    /// Retrieves a Control representing the UI for this controller
    fn ui(&self) -> BindRef<Control> {
        self.ui.clone()
    }
}

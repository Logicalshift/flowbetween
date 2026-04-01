use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;

use std::sync::*;

///
/// Controller that displays the logo for the file chooser
///
pub struct FloLogoController {
    images: Arc<ResourceManager<Image>>,
    ui:     BindRef<Control>
}

impl FloLogoController {
    ///
    /// Creates a new logo controller
    ///
    pub fn new() -> FloLogoController {
        let images  = Arc::new(ResourceManager::new());
        let logo    = images.register(png_static(include_bytes!("../png/intro.png")));
        images.assign_name(&logo, "Logo");

        let ui      = Self::ui(Arc::clone(&images));

        FloLogoController {
            images, ui
        }
    }

    ///
    /// Creates the UI representing the logo
    ///
    fn ui(images: Arc<ResourceManager<Image>>) -> BindRef<Control> {
        let logo = images.get_named_resource("Logo");

        let ui = bind(Control::container()
            .with(vec![
                Control::empty()
                    .with(Bounds::next_vert(8.0)),
                Control::empty()
                    .with(logo)
                    .with(Bounds::stretch_vert(1.0)),
                Control::empty()
                    .with(Bounds::next_vert(4.0)),
                Control::label()
                    .with("F L O W B E T W E E N")
                    .with(FontWeight::Light)
                    .with(TextAlign::Center)
                    .with(Font::Size(28.0))
                    .with(Appearance::Foreground(Color::Rgba(1.0, 1.0, 1.0, 0.8)))
                    .with(Bounds::next_vert(32.0)),
                Control::empty()
                    .with(Bounds::next_vert(16.0))
            ])
            .with(Bounds::fill_all()));

        BindRef::from(ui)
    }
}

impl Controller for FloLogoController {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(Arc::clone(&self.images))
    }
}

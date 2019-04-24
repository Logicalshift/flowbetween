use super::overlay_layers::*;
use super::canvas_renderer::*;
use super::super::model::*;

use flo_ui::*;
use flo_canvas::*;

use std::iter;
use std::sync::*;

///
/// The onion skin renderer deals with rendering the onion skin overlay layers
///
pub struct OnionSkinRenderer {

}

impl OnionSkinRenderer {
    ///
    /// Creates a new onion skin renderer
    ///
    pub fn new() -> OnionSkinRenderer {
        OnionSkinRenderer {

        }
    }

    ///
    /// Performs onion skin rendering on a canvas
    ///
    pub fn render(&self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer, onion_skins: Vec<(OnionSkinTime, Arc<Vec<Draw>>)>, past_color: Color, future_color: Color) {
        if onion_skins.len() == 0 {
            renderer.overlay(canvas, OVERLAY_ONIONSKINS, vec![Draw::ClearCanvas]);
        } else {
            // Onion skins further away in time are less opaque
            let min_opacity     = 0.1;
            let max_opacity     = 0.5;
            let opacity_step    = (max_opacity - min_opacity)/(onion_skins.len() as f64);

            // Generate drawing instructions for each set of onion skins, in reverse order (from least opaque to most opaque)
            let draw_onion_skins = iter::once(Draw::ClearCanvas)
                .chain(onion_skins.into_iter()
                    .rev()
                    .enumerate()
                    .flat_map(|(index, (time, drawing))| {
                        // Work out the colour and opacity of this item
                        let opacity = (index as f64)*opacity_step + min_opacity;
                        let color   = match time {
                            OnionSkinTime::BeforeFrame(_)   => past_color,
                            OnionSkinTime::AfterFrame(_)    => future_color
                        };
                        let color   = color.with_alpha(opacity as f32);

                        iter::once(Draw::NewPath)
                            .chain(drawing.iter()
                                .map(|draw| *draw))
                            .chain(vec![
                                Draw::FillColor(color),
                                Draw::Fill,
                                Draw::LineWidthPixels(2.0),
                                Draw::StrokeColor(Color::Rgba(1.0, 1.0, 1.0, opacity as f32)),
                                Draw::Stroke,
                                Draw::LineWidthPixels(0.5),
                                Draw::StrokeColor(color.with_alpha((opacity+0.1) as f32)),
                                Draw::Stroke
                            ])
                            .collect::<Vec<_>>()
                    }))
                .collect();
            renderer.overlay(canvas, OVERLAY_ONIONSKINS, draw_onion_skins);
        }
    }
}

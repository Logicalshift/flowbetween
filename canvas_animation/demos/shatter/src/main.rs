use flo_draw::*;
use flo_draw::canvas::*;
use flo_canvas_animation::*;
use flo_canvas_animation::effects::*;

use std::thread;
use std::time::{Duration};

fn main() {
    with_2d_graphics(|| {
        // Set up the initial canvas
        let canvas = create_drawing_window("Shattering circle");

        canvas.draw(|gc| {
            gc.clear_canvas(Color::Rgba(0.7, 0.8, 0.5, 1.0));
            gc.canvas_height(1000.0);
            gc.center_region(0.0, 0.0, 1000.0, 1000.0);

            gc.layer(LayerId(2));
            gc.new_path();
            gc.circle(500.0, 500.0, 75.0);

            gc.fill_color(Color::Rgba(0.9, 0.9, 1.0, 1.0));
            gc.line_width(6.0);
            gc.stroke_color(Color::Rgba(0.1, 0.1, 0.1, 1.0));
            gc.fill();
            gc.stroke();
        });

        // Fill a canvas layer with a circle and some regions
        let mut circle_drawing = vec![];

        circle_drawing.circle(500.0, 500.0, 116.0);
        circle_drawing.fill_color(Color::Rgba(0.1, 0.1, 0.1, 1.0));
        circle_drawing.fill();
        circle_drawing.circle(500.0, 500.0, 100.0);
        circle_drawing.fill_color(Color::Rgba(0.91, 1.0, 0.99, 1.0));
        circle_drawing.fill();

        // Create an animation layer for our circle
        let mut animation_layer = AnimationLayer::new();
        animation_layer.draw(circle_drawing);

        // TODO: create a bunch of regions and 'shatter' the circle
    });
}

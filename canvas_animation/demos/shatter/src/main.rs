use flo_draw::*;
use flo_draw::canvas::*;
use flo_curves::bezier::path::*;
use flo_canvas_animation::*;
use flo_canvas_animation::effects::*;

use futures::executor;

use std::f64;
use std::thread;
use std::time::{Duration, Instant};

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

        circle_drawing.new_path();
        circle_drawing.circle(500.0, 500.0, 116.0);
        circle_drawing.fill_color(Color::Rgba(0.1, 0.1, 0.1, 1.0));
        circle_drawing.fill();
        circle_drawing.new_path();
        circle_drawing.circle(500.0, 500.0, 100.0);
        circle_drawing.fill_color(Color::Rgba(0.91, 1.0, 0.99, 1.0));
        circle_drawing.fill();

        // Create an animation layer for our circle
        let mut animation_layer = AnimationLayer::new();
        animation_layer.draw(circle_drawing);

        // Create a bunch of regions to 'shatter' the circle
        for slice_idx in 0..16 {
            // Angle in radians of this slice
            let middle_angle            = f64::consts::PI*2.0 / 16.0 * (slice_idx as f64);
            let start_angle             = middle_angle - (f64::consts::PI*2.0 / 32.0);
            let end_angle               = middle_angle + (f64::consts::PI*2.0 / 32.0);

            // Create a triangle slice
            let (center_x, center_y)    = (500.0, 500.0);
            let (x1, y1)                = (center_x + (f64::sin(start_angle) * 300.0),  center_y + (f64::cos(start_angle) * 300.0));
            let (x2, y2)                = (center_x + (f64::sin(end_angle) * 300.0),    center_y + (f64::cos(end_angle) * 300.0));
            let (x3, y3)                = (center_x + (f64::sin(start_angle) * 16.0),  center_y + (f64::cos(start_angle) * 16.0));
            let (x4, y4)                = (center_x + (f64::sin(end_angle) * 16.0),    center_y + (f64::cos(end_angle) * 16.0));

            let fragment                = BezierPathBuilder::<SimpleBezierPath>::start(Coord2(x3, y3))
                .line_to(Coord2(x1, y1))
                .line_to(Coord2(x2, y2))
                .line_to(Coord2(x4, y4))
                .line_to(Coord2(x3, y3))
                .build();

            // Create an animation effect
            let (dx, dy)                = (f64::sin(middle_angle) * 300.0, f64::cos(middle_angle) * 300.0);
            let motion_effect           = MotionEffect::from_points(Duration::from_secs(20), 
                Coord2(center_x, center_y), 
                vec![
                    (Coord2(center_x + dx * 0.33, center_y + dy * 0.33), Coord2(center_x + dx * 0.66, center_y + dy * 0.66), Coord2(center_x + dx, center_y + dy)),
                    (Coord2(center_x + dx * 0.66, center_y + dy * 0.66), Coord2(center_x + dx * 0.33, center_y + dy * 0.33), Coord2(center_x, center_y))
                ]);

            // Apply a time curve
            let motion_effect           = TimeCurveEffect::with_control_points(motion_effect, vec![(0.0, 10000.0, 10000.0), (10000.0, 19000.0, 20000.0)]);
            let motion_effect           = RepeatEffect::repeat_effect(motion_effect, Duration::from_secs(20));

            // Apply it to a region of the layer
            let motion_effect           = motion_effect.with_region(vec![fragment]);
            animation_layer.add_region(motion_effect);
        }

        // Animate the layer over time
        let start_time                  = Instant::now();
        loop {
            let time_since_start        = start_time.elapsed();
            let animation_layer         = &mut animation_layer;

            // Draw a frame
            canvas.draw(|gc| {
                gc.layer(LayerId(2));
                gc.clear_layer();

                let shatter_drawing = executor::block_on(async move {
                    animation_layer.render_at_time(time_since_start).await
                });

                shatter_drawing.into_iter()
                    .for_each(|draw| gc.draw(draw));
            });

            // Wait for the next frame
            thread::sleep(Duration::from_nanos(1_000_000_000 / 60));
        }
    });
}

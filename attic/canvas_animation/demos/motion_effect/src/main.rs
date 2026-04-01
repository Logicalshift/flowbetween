use flo_draw::*;
use flo_draw::canvas::*;
use flo_stream::*;
use flo_curves::bezier::*;
use flo_canvas_animation::effects::*;

use futures::stream;
use futures::executor;
use futures::prelude::*;
use futures_timer::*;

use std::time::{Duration, Instant};

fn main() {
    with_2d_graphics(|| {
        let (canvas, events) = create_drawing_window_with_events("Drag to create a motion path");

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

        // Simple drag-to-create effect routine
        executor::block_on(async move {
            enum MotionEvent {
                Tick,
                DrawEvent(DrawEvent)
            }

            let start_time  = Instant::now();

            // Add a timer for the animation to the usual draw event stream
            let events              = events.map(|evt| MotionEvent::DrawEvent(evt));
            let tick_stream         = tick_stream().map(|_evt| MotionEvent::Tick);
            let mut events          = stream::select(events, tick_stream);

            // The motion path is the path drawn by the user in the window
            let mut motion_path     = vec![];
            let mut motion_effect   = None;

            while let Some(event) = events.next().await {
                match event {
                    MotionEvent::DrawEvent(DrawEvent::Pointer(PointerAction::ButtonDown, _, initial_state)) => {
                        let initial_point       = initial_state.location_in_canvas.unwrap();
                        let mut motion_points   = vec![Coord2(initial_point.0, initial_point.1)];

                        // Allow the user to draw a path
                        while let Some(event) = events.next().await {
                            match event {
                                MotionEvent::DrawEvent(DrawEvent::Pointer(PointerAction::Drag, _, drag_state)) => {
                                    // Add to the path
                                    let new_point   = drag_state.location_in_canvas.unwrap();
                                    motion_points.push(Coord2(new_point.0, new_point.1));

                                    // Fit to the curve
                                    let fit_path    = fit_curve::<Curve<Coord2>>(&motion_points, 2.0).unwrap_or(vec![]);

                                    // Draw a preview of the path
                                    canvas.draw(|gc| {
                                        gc.layer(LayerId(3));
                                        gc.clear_layer();

                                        gc.push_state();

                                        gc.line_width(2.0);
                                        gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 0.7));
                                        gc.new_dash_pattern();
                                        gc.dash_length(4.0);
                                        gc.dash_length(4.0);

                                        if fit_path.len() > 0 {
                                            gc.new_path();

                                            gc.move_to(fit_path[0].start_point().x() as _, fit_path[0].start_point().y() as _);

                                            for curve in fit_path.iter() {
                                                let (cp1, cp2)  = curve.control_points();
                                                let end_point   = curve.end_point();

                                                gc.bezier_curve_to(end_point.x() as _, end_point.y() as _, cp1.x() as _, cp1.y() as _, cp2.x() as _, cp2.y() as _);
                                            }

                                            gc.stroke();
                                        }

                                        gc.pop_state();
                                    });
                                },

                                MotionEvent::DrawEvent(DrawEvent::Pointer(PointerAction::ButtonUp, _, _)) => {
                                    // Finish the path and replace the motion
                                    motion_path = fit_curve::<Curve<Coord2>>(&motion_points, 2.0).unwrap_or(vec![]);

                                    if motion_path.len() > 0 {
                                        motion_effect = Some(LinearMotionEffect::from_points(Duration::from_secs(10), motion_path[0].start_point(), 
                                            motion_path.iter().map(|curve| {
                                                let (cp1, cp2)  = curve.control_points();
                                                let end_point   = curve.end_point();
                                                (cp1, cp2, end_point)
                                            }).collect()));
                                    } else {
                                        motion_effect = None;
                                    }

                                    // Draw the preview curve
                                    canvas.draw(|gc| {
                                        // Clear the user layer
                                        gc.layer(LayerId(3));
                                        gc.clear_layer();

                                        // Draw the path to the canvas
                                        gc.layer(LayerId(1));
                                        gc.clear_layer();

                                        if motion_path.len() > 0 {
                                            gc.line_width(4.0);
                                            gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));

                                            gc.new_path();

                                            gc.move_to(motion_path[0].start_point().x() as _, motion_path[0].start_point().y() as _);

                                            for curve in motion_path.iter() {
                                                let (cp1, cp2)  = curve.control_points();
                                                let end_point   = curve.end_point();

                                                gc.bezier_curve_to(end_point.x() as _, end_point.y() as _, cp1.x() as _, cp1.y() as _, cp2.x() as _, cp2.y() as _);
                                            }

                                            gc.stroke();

                                            gc.line_width(1.0);
                                            gc.stroke_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
                                            gc.fill_color(Color::Rgba(1.0, 1.0, 1.0, 0.9));

                                            gc.new_path();
                                            gc.circle(motion_path[0].start_point().x() as _, motion_path[0].start_point().y() as _, 6.0);
                                            gc.fill();
                                            gc.stroke();

                                            for curve in motion_path.iter() {
                                                let end_point   = curve.end_point();

                                                gc.new_path();
                                                gc.circle(end_point.x() as _, end_point.y() as _, 6.0);
                                                gc.fill();
                                                gc.stroke();
                                            }
                                        }
                                    });
                                    break;
                                },

                                MotionEvent::DrawEvent(DrawEvent::Pointer(PointerAction::Cancel, _, _)) => {
                                    // Clear the drag path and give up
                                    canvas.draw(|gc| {
                                        gc.layer(LayerId(3));
                                        gc.clear_layer();
                                    });
                                    break;
                                },

                                _ => {
                                    // Unhandled event
                                }
                            }
                        }
                    }

                    MotionEvent::Tick => {
                        if let Some(motion_effect) = &motion_effect {
                            // Update the drawing along the current effect line
                            let time_since_start    = start_time.elapsed().as_millis() as f64;
                            let animation_time      = time_since_start % 10_000.0;
                            let start_point         = motion_path[0].start_point();
                            let offset              = motion_effect.offset_at_time(animation_time, 0.01);
                            let pos                 = start_point + offset;

                            canvas.draw(|gc| {
                                gc.layer(LayerId(2));
                                gc.clear_layer();
                                gc.new_path();
                                gc.circle(pos.x() as _, pos.y() as _, 75.0);

                                gc.fill_color(Color::Rgba(0.9, 0.9, 1.0, 0.6));
                                gc.line_width(6.0);
                                gc.stroke_color(Color::Rgba(0.1, 0.1, 0.1, 1.0));
                                gc.fill();
                                gc.stroke();

                                gc.line_width(1.0);
                                gc.stroke_color(Color::Rgba(0.5, 0.1, 0.1, 1.0));

                                gc.new_path();
                                gc.move_to((pos.x() - 40.0) as _, pos.y() as _);
                                gc.line_to((pos.x() + 40.0) as _, pos.y() as _);
                                gc.move_to(pos.x() as _, (pos.y() - 40.0) as _);
                                gc.line_to(pos.x() as _, (pos.y() + 40.0) as _);
                                gc.stroke();
                            });
                        }
                    }

                    _ => { 
                        // Unhandled event
                    }
                }
            }
        });
    });
}

///
/// Generates ticks to advance the animation
///
fn tick_stream() -> impl Send+Unpin+Stream<Item=()> {
    generator_stream(|yield_value| async move {
        // Set up the clock
        let start_time          = Instant::now();
        let mut last_time       = Duration::from_millis(0);

        // We limit to a certain number of ticks per callback (in case the task is suspended or stuck for a prolonged period of time)
        let max_ticks_per_call  = 5;

        // Ticks are generated 60 times a second
        let tick_length         = Duration::from_nanos(1_000_000_000 / 60);

        loop {
            // Time that has elapsed since the last tick
            let elapsed         = start_time.elapsed() - last_time;

            // Time remaining
            let mut remaining   = elapsed;
            let mut num_ticks   = 0;
            while remaining >= tick_length {
                if num_ticks < max_ticks_per_call {
                    // Generate the tick
                    yield_value(()).await;
                    num_ticks += 1;
                }

                // Remove from the remaining time, and update the last tick time
                remaining -= tick_length;
                last_time += tick_length;
            }

            // Wait for half a tick before generating more ticks
            let next_time = tick_length - remaining;
            let wait_time = Duration::min(tick_length / 2, next_time);

            Delay::new(wait_time).await;
        }
    }.boxed())
}

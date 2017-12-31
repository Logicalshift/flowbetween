use desync::*;
use canvas::*;
use binding::*;
use binding::binding_context::*;

use std::sync::*;
use std::mem;
use std::ops::Deref;

///
/// The binding canvas is a canvas that can have an attached rendering
/// function. It will invalidate itself if any bindings used in that
/// rendering function are changed.
/// 
pub struct BindingCanvas {
    /// The canvas that this is bound to (we use an Arc<> so the same canvas can be re-used elsewhere)
    canvas: Arc<Canvas>,

    /// The core stores the binding canvas data for this item
    core: Arc<Desync<BindingCanvasCore>>
}

///
/// Core data for the binding canvas
/// 
struct BindingCanvasCore {
    /// True if the canvas has been invalidated
    invalidated: bool,

    /// The drawing function, or none if there is no drawing function
    draw_fn: Option<Box<Fn(&mut GraphicsPrimitives) -> ()+Send+Sync>>,

    /// The notifications that are currently active for this core
    active_notifications: Option<Box<Releasable>>
}

///
/// Just a wrapper for a weak ref to the binding canvas core
/// 
struct CoreNotifiable(Weak<Desync<BindingCanvasCore>>);

impl BindingCanvasCore {
    ///
    /// Marks the notifications associated with this object as done
    /// 
    fn done_with_notifications(&mut self) {
        // Swap out the notifications
        let mut notifications = None;
        mem::swap(&mut notifications, &mut self.active_notifications);

        // Mark as done if there are any
        if let Some(mut notifications) = notifications {
            notifications.done();
        }
    }

    ///
    /// Redraws a canvas core and rebinds the notifications
    /// 
    fn redraw_and_notify_if_invalid(core_ref: &Arc<Desync<BindingCanvasCore>>, canvas: &Canvas) {
        // Create a weak reference to the core (which is what we'll notify)
        let weak_core = CoreNotifiable(Arc::downgrade(core_ref));

        core_ref.sync(move |core| {
            if core.invalidated {
                // Finish any active notifications
                core.done_with_notifications();

                // Redraw and notify
                let notification_lifetime = core.redraw(canvas, Arc::new(weak_core));
                core.active_notifications = Some(notification_lifetime);
            }
        });
    }

    ///
    /// Redraws the content of this core on a canvas and sets the bindings to notify the specified object
    /// 
    fn redraw(&mut self, canvas: &Canvas, to_notify: Arc<Notifiable>) -> Box<Releasable> {
        let mut release_notifications: Box<Releasable> = Box::new(vec![]);
        let draw_fn = &self.draw_fn;

        // Draw to the canvas
        canvas.draw(|gc| {
            // We always start with a clear
            gc.clear_canvas();

            // Call the drawing function in a binding context
            let (_result, deps) = BindingContext::bind(move || {
                if let &Some(ref draw_fn) = draw_fn {
                    draw_fn(gc);
                }
            });

            // Cause a notification when the binding changes
            release_notifications = deps.when_changed(to_notify);
        });

        // No longer invalidated
        self.invalidated = false;

        // Result is the notifications to be released
        release_notifications
    }
}

impl Drop for BindingCanvasCore {
    fn drop(&mut self) {
        self.done_with_notifications();
    }
}

impl Notifiable for CoreNotifiable {
    fn mark_as_changed(&self) {
        // If the reference is still active, reconstitute the core and set it to invalid
        if let Some(to_notify) = self.0.upgrade() {
            to_notify.async(|core| {
                core.invalidated = true
            });
        }
    }
}

impl BindingCanvas {
    ///
    /// Creates a new BindingCanvas
    /// 
    pub fn new() -> BindingCanvas {
        Self::from(Arc::new(Canvas::new()))
    }

    ///
    /// Creates a new binding canvas from a canvas
    /// 
    pub fn from(canvas: Arc<Canvas>) -> BindingCanvas {
        let core = BindingCanvasCore {
            invalidated:            false,
            draw_fn:                None,
            active_notifications:   None
        };

        BindingCanvas {
            canvas: canvas,
            core:   Arc::new(Desync::new(core))
        }
    }

    ///
    /// Creates a new BindingCanvas with a drawing function
    /// 
    pub fn with_drawing<DrawingFn: 'static+Fn(&mut GraphicsPrimitives) -> ()+Send+Sync>(&self, draw: DrawingFn) -> BindingCanvas {
        let canvas = Self::new();
        canvas.on_redraw(draw);

        canvas
    }

    ///
    /// Sets the drawing function for the canvas
    /// 
    /// Canvases don't have a drawing function by default, so it's safe
    /// to draw directly on them as they'll never become invalidated.
    /// Once a drawing function is set, any bindings it may have will
    /// cause it to become invalidated if they change. Additionally,
    /// setting a drawing function will invalidate the canvas.
    /// 
    pub fn on_redraw<DrawingFn: 'static+Fn(&mut GraphicsPrimitives) -> ()+Send+Sync>(&self, draw: DrawingFn) {
        self.core.async(move |core| {
            core.done_with_notifications();

            core.invalidated    = true;
            core.draw_fn        = Some(Box::new(draw));
        });
    }

    ///
    /// Redraws the canvas if it is marked as invalid
    /// 
    pub fn redraw_if_invalid(&self) {
        let canvas = &*self.canvas;
        BindingCanvasCore::redraw_and_notify_if_invalid(&self.core, canvas);
    }

    ///
    /// Marks this canvas as invalidated (will be redrawn on the next request)
    /// 
    pub fn invalidate(&self) {
        self.core.async(|core| core.invalidated = true);
    }
}

impl Deref for BindingCanvas {
    type Target = Canvas;

    fn deref(&self) -> &Canvas {
        &*self.canvas
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::executor;

    #[test]
    fn binding_canvas_works_like_canvas() {
        let canvas      = BindingCanvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        
        // Draw using a graphics context
        canvas.draw(|gc| {
            gc.new_path();
        });

        // Check we can get the results via the stream
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));
    }

    #[test]
    fn will_invalidate_and_redraw_when_function_assigned() {
        let canvas      = BindingCanvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        
        // Set a bound function
        canvas.set_drawing_fn(|gc| {
            gc.new_path();
        });

        // Redraw so it gets called
        canvas.redraw_if_invalid();

        // Check we can get the results via the stream
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));
    }

    #[test]
    fn redraws_when_binding_changes() {
        let mut binding = bind((1.0, 2.0));
        let canvas      = BindingCanvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        
        // Set a bound function
        let draw_binding = binding.clone();
        canvas.set_drawing_fn(move |gc| {
            let (x, y) = draw_binding.get();

            gc.new_path();
            gc.move_to(x, y);
        });

        // Redraw so it gets called
        canvas.redraw_if_invalid();

        // Should draw the first set of functions
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));
        assert!(stream.wait_stream() == Some(Ok(Draw::Move(1.0, 2.0))));

        // Update the binding
        binding.set((4.0, 5.0));

        // Redraw with the updated binding
        canvas.redraw_if_invalid();

        // Should redraw the canvas now
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));
        assert!(stream.wait_stream() == Some(Ok(Draw::Move(4.0, 5.0))));
    }
}

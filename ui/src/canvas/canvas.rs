use super::gc::*;
use super::draw::*;
use super::color::*;
use super::transform2d::*;

use std::sync::*;
use std::mem;

use desync::*;
use futures::task::Task;
use futures::task;
use futures::{Stream,Poll,Async};

///
/// The core structure used to store details of a canvas 
///
struct CanvasCore {
    /// What was drawn since the last clear command was sent to this canvas
    drawing_since_last_clear: Vec<Draw>,

    // The number of times the canvas has been cleared
    clear_count: u32,

    // Tasks to notify next time we add to the canvas
    pending_tasks: Vec<Task>,

    /// True if the canvas has been dropped
    dropped: bool
}

///
/// A canvas is an abstract interface for drawing graphics. It doesn't actually provide a means to
/// render anything, but rather a way to describe how things should be drawn and pass those on to
/// a renderer elsewhere. 
///
pub struct Canvas {
    /// The core is shared amongst the canvas streams as well as used by the canvas itself
    core: Arc<Desync<CanvasCore>>
}

impl CanvasCore {
    fn write(&mut self, to_draw: Vec<Draw>) {
        let mut pending_tasks   = vec![];

        // Get the tasks we're going to notify about the new command
        mem::swap(&mut self.pending_tasks, &mut pending_tasks);

        // Process the drawing commands
        to_draw.iter().for_each(|draw| {
            // Clearing the canvas empties the command list and updates the clear count
            if let &Draw::ClearCanvas = draw {
                self.drawing_since_last_clear   = vec![];
                self.clear_count                = self.clear_count.wrapping_add(1);
            }

            // Add the command to the drawing list (there's always a clear at the start)
            self.drawing_since_last_clear.push(*draw);
        });

        // Notify the pending tasks that there are new drawing commands available
        pending_tasks.iter_mut().for_each(|task| task.notify());
    }
}

impl Canvas {
    ///
    /// Creates a new, blank, canvas
    ///
    pub fn new() -> Canvas {
        // A canvas is initially just a clear command
        let core = CanvasCore { 
            drawing_since_last_clear:   vec![ Draw::ClearCanvas ],
            clear_count:                0,
            pending_tasks:              vec![ ],
            dropped:                    false
        };

        Canvas {
            core: Arc::new(Desync::new(core))
        }
    }

    ///
    /// Sends some new drawing commands to this canvas
    ///
    pub fn write(&mut self, to_draw: Vec<Draw>) {
        // Only draw if there are any drawing commands
        if to_draw.len() != 0 {
            self.core.async(move |core| core.write(to_draw));
        }
    }

    ///
    /// Provides a way to draw on this canvas via a GC
    /// 
    pub fn draw<FnAction>(&self, action: FnAction)
    where FnAction: Send+FnOnce(&mut GraphicsContext) -> () {
        self.core.sync(move |core| {
            let mut graphics_context = CoreContext {
                core:       core,
                pending:    vec![]
            };

            action(&mut graphics_context);
        })
    }

    ///
    /// Creates a stream for reading the instructions from this canvas
    ///
    pub fn stream(&self) -> Box<Stream<Item=Draw,Error=()>> {
        Box::new(CanvasStream { 
            core:               self.core.clone(),
            pos:                0,
            active_clear_count: 0
        })
    }

    ///
    /// Retrieves the list of drawing actions in this canvas
    ///
    pub fn get_drawing(&self) -> Vec<Draw> {
        self.core.sync(|core| core.drawing_since_last_clear.clone())
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        self.core.async(|core| {
            let mut pending_tasks   = vec![];

            // Get the tasks we're going to notify about the new command
            mem::swap(&mut core.pending_tasks, &mut pending_tasks);

            // Mark the core as dropped
            core.dropped = true;

            // Notify any tasks that are using the canvas that it has gone away
            pending_tasks.iter_mut().for_each(|task| task.notify());
        });
    }
}

///
/// Graphics context built from a canvas core
///
struct CoreContext<'a> {
    core:       &'a mut CanvasCore,
    pending:    Vec<Draw>
}

impl<'a> GraphicsContext for CoreContext<'a> {
    fn new_path(&mut self)                          { self.pending.push(Draw::NewPath); }
    fn move_to(&mut self, x: f32, y: f32)           { self.pending.push(Draw::Move(x, y)); }
    fn line_to(&mut self, x: f32, y: f32)           { self.pending.push(Draw::Line(x, y)); }

    fn bezier_curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.pending.push(Draw::BezierCurve((x1, y1), (x2, y2), (x3, y3)));
    }

    fn rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.pending.push(Draw::Rect((x1, y1), (x2, y2)));
    }

    fn fill(&mut self)                              { self.pending.push(Draw::Fill); }
    fn stroke(&mut self)                            { self.pending.push(Draw::Stroke); }
    fn line_width(&mut self, width: f32)            { self.pending.push(Draw::LineWidth(width)); }
    fn line_join(&mut self, join: LineJoin)         { self.pending.push(Draw::LineJoin(join)); }
    fn line_cap(&mut self, cap: LineCap)            { self.pending.push(Draw::LineCap(cap)); }
    fn dash_length(&mut self, length: f32)          { self.pending.push(Draw::DashLength(length)); }
    fn dash_offset(&mut self, offset: f32)          { self.pending.push(Draw::DashOffset(offset)); }
    fn fill_color(&mut self, col: Color)            { self.pending.push(Draw::FillColor(col)); }
    fn stroke_color(&mut self, col: Color)          { self.pending.push(Draw::StrokeColor(col)); }
    fn blend_mode(&mut self, mode: BlendMode)       { self.pending.push(Draw::BlendMode(mode)); }
    fn identity_transform(&mut self)                { self.pending.push(Draw::IdentityTransform); }
    fn canvas_height(&mut self, height: f32)        { self.pending.push(Draw::CanvasHeight(height)); }
    fn transform(&mut self, transform: Transform2D) { self.pending.push(Draw::MultiplyTransform(transform)); }
    fn unclip(&mut self)                            { self.pending.push(Draw::Unclip); }
    fn clip(&mut self)                              { self.pending.push(Draw::Clip); }
    fn store(&mut self)                             { self.pending.push(Draw::Store); }
    fn restore(&mut self)                           { self.pending.push(Draw::Restore); }
    fn push_state(&mut self)                        { self.pending.push(Draw::PushState); }
    fn pop_state(&mut self)                         { self.pending.push(Draw::PopState); }
    fn clear_canvas(&mut self)                      { self.pending.push(Draw::ClearCanvas); }
}

impl<'a> Drop for CoreContext<'a> {
    fn drop(&mut self) {
        let mut to_draw = vec![];
        mem::swap(&mut self.pending, &mut to_draw);
        self.core.write(to_draw);
    }
}

///
/// The canvas stream can be used to read the contents of the canvas and follow new content as it arrives.
/// Unconsumed commands are cut off if the `Draw::ClearCanvas` command is issued.
///
pub struct CanvasStream {
    /// The core is shared amongst the canvas streams as well as used by the canvas itself
    core: Arc<Desync<CanvasCore>>,

    /// Position in the canvas command list
    pos: usize,

    /// The clear count from the canvas (we reset the position if this doesn't match)
    active_clear_count: u32
}

impl Stream for CanvasStream {
    type Item = Draw;
    type Error = ();

   fn poll(&mut self) -> Poll<Option<Draw>, ()> {
        use self::Async::*;

        let active_clear_count  = &mut self.active_clear_count;
        let pos                 = &mut self.pos;

        self.core.sync(move |core| {
            if core.clear_count != *active_clear_count {
                // The canvas has been cleared since the last read, so reset the position back to the beginning
                *active_clear_count = core.clear_count;
                *pos                = 0;
            }

            if *pos < core.drawing_since_last_clear.len() {
                // There are still values in the canvas that we haven't returned yet
                let value   = core.drawing_since_last_clear[*pos];
                *pos        = *pos+1;

                Ok(Ready(Some(value)))
            } else if core.dropped {
                // Once the core is dropped, the canvas stream is finished
                Ok(Ready(None))
            } else {
                // Need to be notified when the canvas changes
                core.pending_tasks.push(task::current());

                Ok(NotReady)
            }
        })
   }
}

#[cfg(test)]
mod test {
    use super::*;

    use futures::executor;

    use std::thread::*;
    use std::time::*;

    #[test]
    fn can_draw_to_canvas() {
        let mut canvas = Canvas::new();

        canvas.write(vec![Draw::NewPath]);
    }

    #[test]
    fn can_follow_canvas_stream() {
        let mut canvas  = Canvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        
        // Thread to draw some stuff to the canvas
        spawn(move || {
            sleep(Duration::from_millis(50));

            canvas.write(vec![
                Draw::NewPath,
                Draw::Move(0.0, 0.0),
                Draw::Line(10.0, 0.0),
                Draw::Line(10.0, 10.0),
                Draw::Line(0.0, 10.0)
            ]);
        });

        // TODO: if the canvas fails to notify, this will block forever :-/

        // Check we can get the results via the stream
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));
        assert!(stream.wait_stream() == Some(Ok(Draw::Move(0.0, 0.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(10.0, 0.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(10.0, 10.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(0.0, 10.0))));

        // When the thread goes away, it'll drop the canvas, so we should get the 'None' request here too
        assert!(stream.wait_stream() == None);
    }

    #[test]
    fn can_draw_using_gc() {
        let canvas      = Canvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        
        // Draw using a graphics context
        canvas.draw(|gc| {
            gc.new_path();
            gc.move_to(0.0, 0.0);
            gc.line_to(10.0, 0.0);
            gc.line_to(10.0, 10.0);
            gc.line_to(0.0, 10.0);
        });

        // Check we can get the results via the stream
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));
        assert!(stream.wait_stream() == Some(Ok(Draw::Move(0.0, 0.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(10.0, 0.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(10.0, 10.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(0.0, 10.0))));
    }

    #[test]
    fn can_follow_many_streams() {
        let mut canvas  = Canvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        let mut stream2  = executor::spawn(canvas.stream());
        
        // Thread to draw some stuff to the canvas
        spawn(move || {
            sleep(Duration::from_millis(50));

            canvas.write(vec![
                Draw::NewPath,
                Draw::Move(0.0, 0.0),
                Draw::Line(10.0, 0.0),
                Draw::Line(10.0, 10.0),
                Draw::Line(0.0, 10.0)
            ]);
        });

        // TODO: if the canvas fails to notify, this will block forever :-/

        // Check we can get the results via the stream
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));
        assert!(stream.wait_stream() == Some(Ok(Draw::Move(0.0, 0.0))));

        assert!(stream2.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream2.wait_stream() == Some(Ok(Draw::NewPath)));
        assert!(stream2.wait_stream() == Some(Ok(Draw::Move(0.0, 0.0))));

        assert!(stream.wait_stream() == Some(Ok(Draw::Line(10.0, 0.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(10.0, 10.0))));
        assert!(stream.wait_stream() == Some(Ok(Draw::Line(0.0, 10.0))));

        assert!(stream2.wait_stream() == Some(Ok(Draw::Line(10.0, 0.0))));
        assert!(stream2.wait_stream() == Some(Ok(Draw::Line(10.0, 10.0))));
        assert!(stream2.wait_stream() == Some(Ok(Draw::Line(0.0, 10.0))));

        // When the thread goes away, it'll drop the canvas, so we should get the 'None' request here too
        assert!(stream.wait_stream() == None);
        assert!(stream2.wait_stream() == None);
    }

    #[test]
    fn commands_after_clear_are_suppressed() {
        let mut canvas  = Canvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        
        // Thread to draw some stuff to the canvas
        spawn(move || {
            sleep(Duration::from_millis(50));

            canvas.write(vec![
                Draw::NewPath,
                Draw::Move(0.0, 0.0),
                Draw::Line(10.0, 0.0),
                Draw::Line(10.0, 10.0),
                Draw::Line(0.0, 10.0)
            ]);

            // Enough time that we read the first few commands
            sleep(Duration::from_millis(100));

            canvas.write(vec![
                Draw::ClearCanvas,
                Draw::Move(200.0, 200.0),
            ]);
        });

        // TODO: if the canvas fails to notify, this will block forever :-/

        // Check we can get the results via the stream
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::NewPath)));

        // Give the thread some time to clear the canvas
        sleep(Duration::from_millis(120));

        // Commands we sent before the flush are gone
        assert!(stream.wait_stream() == Some(Ok(Draw::ClearCanvas)));
        assert!(stream.wait_stream() == Some(Ok(Draw::Move(200.0, 200.0))));

        // When the thread goes away, it'll drop the canvas, so we should get the 'None' request here too
        assert!(stream.wait_stream() == None);
    }
}

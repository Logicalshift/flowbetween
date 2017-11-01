use super::draw::*;

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
    pub fn draw(&mut self, to_draw: Vec<Draw>) {
        // Only draw if there are any drawing commands
        if to_draw.len() != 0 {
            self.core.async(move |core| {
                let mut pending_tasks   = vec![];

                // Get the tasks we're going to notify about the new command
                mem::swap(&mut core.pending_tasks, &mut pending_tasks);

                // Process the drawing commands
                to_draw.iter().for_each(|draw| {
                    // Clearing the canvas empties the command list and updates the clear count
                    if let &Draw::ClearCanvas = draw {
                        core.drawing_since_last_clear   = vec![];
                        core.clear_count                = core.clear_count.wrapping_add(1);
                    }

                    // Add the command to the drawing list (there's always a clear at the start)
                    core.drawing_since_last_clear.push(*draw);
                });

                // Notify the pending tasks that there are new drawing commands available
                pending_tasks.iter_mut().for_each(|task| task.notify());
            });
        }
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

        canvas.draw(vec![Draw::NewPath]);
    }

    #[test]
    fn can_follow_canvas_stream() {
        let mut canvas  = Canvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        
        // Thread to draw some stuff to the canvas
        spawn(move || {
            sleep(Duration::from_millis(50));

            canvas.draw(vec![
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
    fn can_follow_many_streams() {
        let mut canvas  = Canvas::new();
        let mut stream  = executor::spawn(canvas.stream());
        let mut stream2  = executor::spawn(canvas.stream());
        
        // Thread to draw some stuff to the canvas
        spawn(move || {
            sleep(Duration::from_millis(50));

            canvas.draw(vec![
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

            canvas.draw(vec![
                Draw::NewPath,
                Draw::Move(0.0, 0.0),
                Draw::Line(10.0, 0.0),
                Draw::Line(10.0, 10.0),
                Draw::Line(0.0, 10.0)
            ]);

            // Enough time that we read the first few commands
            sleep(Duration::from_millis(100));

            canvas.draw(vec![
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

use super::draw::*;

use std::sync::*;
use std::mem;

use futures::task::*;

///
/// The core structure used to store details of a canvas 
///
struct CanvasCore {
    /// What was drawn since the last clear command was sent to this canvas
    drawing_since_last_clear: Vec<Draw>,

    // The number of times the canvas has been cleared
    clear_count: u32,

    // Tasks to notify next time we add to the canvas
    pending_tasks: Vec<Task>
}

///
/// A canvas is an abstract interface for drawing graphics. It doesn't actually provide a means to
/// render anything, but rather a way to describe how things should be drawn and pass those on to
/// a renderer elsewhere. 
///
pub struct Canvas {
    /// The core is shared amongst the canvas streams as well as used by the canvas itself
    core: Arc<Mutex<CanvasCore>>
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
            pending_tasks:              vec![ ]
        };

        Canvas {
            core: Arc::new(Mutex::new(core))
        }
    }

    ///
    /// Sends some new drawing commands to this canvas
    ///
    pub fn draw(&mut self, to_draw: &[Draw]) {
        // Only draw if there are any drawing commands
        if to_draw.len() != 0 {
            // Draw with the core lock, but notify after the lock is released
            let mut pending_tasks = {
                // Unlock the core
                let mut core            = self.core.lock().unwrap();
                let mut pending_tasks   = vec![];

                // Get the tasks we're going to notify about the new command
                mem::swap(&mut core.pending_tasks, &mut pending_tasks);

                // Process the drawing commands
                to_draw.iter().for_each(|draw| {
                    // Clearing the canvas empties the command list and updates the clear count
                    if let &Draw::ClearCanvas = draw {
                        core.drawing_since_last_clear   = vec![];
                        core.clear_count                = core.clear_count+1;
                    }

                    // Add the command to the drawing list (there's always a clear at the start)
                    core.drawing_since_last_clear.push(*draw);
                });

                // Result is the set of pending tasks
                pending_tasks
            };

            // Tell everything about this task
            pending_tasks.iter_mut().for_each(|task| task.notify());
        }
    }
}

///
/// The canvas stream can be used to read the contents of the canvas and follow new content as it arrives.
/// Unconsumed commands are cut off if the `Draw::ClearCanvas` command is issued.
///
pub struct CanvasStream {

}
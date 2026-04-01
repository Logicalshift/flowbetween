use super::update::*;
use super::super::control::*;
use super::super::controller::*;
use super::super::binding_canvas::*;
use super::super::resource_manager::*;

use flo_canvas::*;
use flo_binding::*;

use futures::*;
use futures::task::{Poll, Context};
use futures::stream::{BoxStream};

use std::pin::*;
use std::sync::*;
use std::collections::HashMap;

///
/// Tracker for an individual canvas
///
struct CanvasStreamTracker {
    /// The stream for the current canvas
    stream: BoxStream<'static, Draw>
}

impl CanvasStreamTracker {
    pub fn new(canvas_resource: &Resource<BindingCanvas>) -> CanvasStreamTracker {
        CanvasStreamTracker {
            stream: Box::pin(canvas_resource.stream())
        }
    }
}

///
/// Provides all updates for all canvases referenced by a controller
///
pub struct CanvasUpdateStream {
    /// The controller that we're providing updates for
    root_controller: Weak<dyn Controller>,

    /// The update streams for the subcontrollers
    sub_controllers: HashMap<String, CanvasUpdateStream>,

    /// The updates for the root controller UI
    controller_updates: FollowStream<Control, BindRef<Control>>,

    /// The canvases that are being tracked at the moment
    canvas_trackers: HashMap<String, CanvasStreamTracker>
}

impl CanvasUpdateStream {
    ///
    /// Creates a new canvas update stream for a controller
    ///
    pub fn new(root_controller: Arc<dyn Controller>) -> CanvasUpdateStream {
        let controller_updates  = follow(root_controller.ui());
        let root_controller     = Arc::downgrade(&root_controller);

        CanvasUpdateStream {
            root_controller:    root_controller,
            controller_updates: controller_updates,
            sub_controllers:    HashMap::new(),
            canvas_trackers:    HashMap::new()
        }
    }

    ///
    /// Updates the set of items that we're tracking for this controller
    ///
    pub fn update_controller_content(&mut self, root_controller: &Arc<dyn Controller>, new_content: &Control) {
        // We regenerate the hashmaps for the subcontrollers and canvases
        let mut new_subcontrollers  = HashMap::new();
        let mut new_canvases        = HashMap::new();

        // Iterate through the control and its child controls
        let mut to_process          = vec![new_content];

        while let Some(next_control) = to_process.pop() {
            // Fetch the control properties
            let canvas          = next_control.canvas_resource();
            let controller_name = next_control.controller();
            let subcomponents   = next_control.subcomponents();

            // Create/keep the canvas tracker for the next canvas
            if let Some(canvas) = canvas {
                // Name is either the assigned name or the ID
                let canvas_name = if let Some(name) = canvas.name() {
                    String::from(name)
                } else {
                    canvas.id().to_string()
                };

                // Keep the existing canvas if there is one
                if new_canvases.contains_key(&canvas_name) {
                    // Already found this canvas
                } else if let Some(existing_canvas) = self.canvas_trackers.remove(&canvas_name) {
                    // Canvas already being tracked in the previous incarnation of this object
                    new_canvases.insert(canvas_name, existing_canvas);
                } else {
                    // Need to create a new canvas tracker
                    let tracker = CanvasStreamTracker::new(canvas);
                    new_canvases.insert(canvas_name, tracker);
                }
            }

            // Create/keep the canvas stream for the next controller
            if let Some(controller_name) = controller_name {
                let controller_name = String::from(controller_name);

                if let Some(controller) = root_controller.get_subcontroller(&controller_name) {
                    if new_subcontrollers.contains_key(&controller_name) {
                        // Already found this controller
                    } else if let Some(existing_controller) = self.sub_controllers.remove(&controller_name) {
                        // Was already tracking this controller
                        new_subcontrollers.insert(controller_name, existing_controller);
                    } else {
                        // Need to create a new controller stream
                        let new_stream = CanvasUpdateStream::new(controller);
                        new_subcontrollers.insert(controller_name, new_stream);
                    }
                }
            }

            // Push the controls to process next
            if let Some(subcomponents) = subcomponents {
                to_process.extend(subcomponents.iter());
            }
        }

        // Update the subcontrollers and canvases
        self.sub_controllers = new_subcontrollers;
        self.canvas_trackers = new_canvases;
    }
}

impl Stream for CanvasUpdateStream {
    type Item = CanvasDiff;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<CanvasDiff>> {
        if let Some(root_controller) = self.root_controller.upgrade() {
            // Poll for control updates
            let mut control_update_poll = self.controller_updates.poll_next_unpin(context);

            while let Poll::Ready(Some(control)) = control_update_poll {
                // Update with the new control
                self.update_controller_content(&root_controller, &control);

                // Poll again
                control_update_poll = self.controller_updates.poll_next_unpin(context);
            }

            if let Poll::Ready(None) = control_update_poll {
                // Subcontroller was deleted, so there are no more updates to process for this subcontroller
                return Poll::Ready(None);
            }

            // Poll each of the subcontrollers to see if they produce a diff
            let mut removed_subcontrollers = vec![];
            for (name, stream) in self.sub_controllers.iter_mut() {
                let subcontroller_poll = stream.poll_next_unpin(context);

                if let Poll::Ready(Some(mut subcontroller_update)) = subcontroller_poll {
                    // Insert the controller name at the start of the path
                    subcontroller_update.controller.insert(0, name.clone());

                    // This is the result of this poll
                    return Poll::Ready(Some(subcontroller_update));
                }

                if let Poll::Ready(None) = subcontroller_poll {
                    removed_subcontrollers.push(name.clone());
                }
            }

            // Try to re-create any removed subcontrollers
            for removed_subcontroller_name in removed_subcontrollers {
                // Remove the old instance of this subcontroller
                self.sub_controllers.remove(&removed_subcontroller_name);

                // If it still exists, recreate it
                if let Some(subcontroller) = root_controller.get_subcontroller(&removed_subcontroller_name) {
                    // Create a new canvas stream from the subcontroller
                    let new_stream = CanvasUpdateStream::new(subcontroller);
                    self.sub_controllers.insert(removed_subcontroller_name, new_stream);

                    // Notify the task immediately to check the new controller for updates
                    context.waker().clone().wake();
                }
            }

            // Poll each of the canvases to see if they have any updates
            let mut removed_canvases = vec![];
            for (canvas_name, tracker) in self.canvas_trackers.iter_mut() {
                let mut updates = vec![];

                let mut canvas_poll = tracker.stream.poll_next_unpin(context);
                while let Poll::Ready(Some(canvas_command)) = canvas_poll {
                    updates.push(canvas_command);

                    canvas_poll = tracker.stream.poll_next_unpin(context);
                }

                if let Poll::Ready(None) = canvas_poll {
                    // Canvas stream has ended
                    removed_canvases.push(canvas_name.clone());
                }

                if updates.len() > 0 {
                    // This generates a canvas diff for this controller
                    let canvas_diff = CanvasDiff {
                        controller:     vec![],
                        canvas_name:    canvas_name.clone(),
                        updates:        updates
                    };

                    return Poll::Ready(Some(canvas_diff));
                }
            }

            // Try to re-create any removed canvases (whose streams have come to an end)
            for removed_canvas_name in removed_canvases {
                // Remove the old instance of this canvas
                self.canvas_trackers.remove(&removed_canvas_name);

                // If it still exists, recreate it
                if let Some(canvas) = root_controller.get_canvas_resources().and_then(|res| res.get_named_resource(&removed_canvas_name)) {
                    // Create a new canvas stream from the subcontroller
                    let new_tracker = CanvasStreamTracker::new(&canvas);
                    self.canvas_trackers.insert(removed_canvas_name, new_tracker);

                    // Notify the task immediately to check the new canvas for updates
                    context.waker().clone().wake();
                }
            }

            // Polled everything and no updates were available
            Poll::Pending
        } else {
            // Root controller has gone, so this stream has no more updates
            Poll::Ready(None)
        }
    }
}

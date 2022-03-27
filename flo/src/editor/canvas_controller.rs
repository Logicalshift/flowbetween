use crate::model::*;
use crate::tools::*;
use crate::animation_canvas::*;
use crate::animation_canvas::overlay_layers::*;

use flo_ui::*;
use flo_canvas::*;
use flo_binding::*;
use flo_animation::*;
use ::desync::*;
use futures::future;
use futures::prelude::*;
use futures::future::{BoxFuture};

use std::sync::*;
use std::time::Duration;
use std::collections::{HashSet};

const MAIN_CANVAS: &str     = "main";
const PAINT_ACTION: &str    = "Paint";

///
/// The core of the canvas
///
struct CanvasCore<Anim: Animation+EditableAnimation> {
    /// The canvas renderer
    renderer: CanvasRenderer,

    /// The animation model
    model: Arc<FloModel<Anim>>,

    /// Set to true if the current tick contains an edit of some kind
    pending_finish_action: bool,

    /// The canvas invalidation count specified in the model when the canvas was drawn
    current_invalidation_count: u64,

    /// Executes actions for the canvas tools
    canvas_tools: CanvasTools<Anim>,

    /// The most recent paint device that received input for this canvas
    last_paint_device: Option<PaintDevice>,

    /// The time of the current frame
    current_time: Duration
}

///
/// The canvas controller manages the main drawing canvas
///
pub struct CanvasController<Anim: Animation+EditableAnimation> {
    ui:                 BindRef<Control>,
    canvases:           Arc<ResourceManager<BindingCanvas>>,
    anim_model:         Arc<FloModel<Anim>>,
    tool_changed:       Arc<Mutex<bool>>,
    _onion_skin_model:  BindRef<(Color, Color, Vec<(OnionSkinTime, Arc<Vec<Draw>>)>)>,

    core:               Arc<Desync<CanvasCore<Anim>>>
}

impl<Anim: Animation+EditableAnimation+'static> CanvasController<Anim> {
    ///
    /// Creates a new canvas controller
    ///
    pub fn new(view_model: &FloModel<Anim>) -> CanvasController<Anim> {
        // Create the resources
        let canvases            = ResourceManager::new();

        let view_model          = Arc::new(view_model.clone());
        let renderer            = CanvasRenderer::new();
        let canvas_tools        = CanvasTools::from_model(&*view_model);
        let main_canvas         = Self::create_main_canvas(&canvases);
        let ui                  = Self::ui(main_canvas.clone(), view_model.size.clone());
        let tool_changed        = Arc::new(Mutex::new(true));
        let onion_skin_model    = Self::onion_skin_binding(&*view_model);

        // Set the tool changed flag whenever the effective tool changes
        // Note: the keep_alive() here will leak if the controller lives for less time than the model
        let also_tool_changed = Arc::clone(&tool_changed);
        view_model.tools().effective_tool
            .when_changed(notify(move || { *also_tool_changed.lock().unwrap() = true; }))
            .keep_alive();

        // Create the core to perform the actual rendering
        let core                = Desync::new(CanvasCore {
                renderer:                   renderer,
                model:                      view_model.clone(),
                pending_finish_action:      false,
                canvas_tools:               canvas_tools,
                last_paint_device:          None,
                current_time:               Duration::new(0, 0),
                current_invalidation_count: 0
            });
        let core                = Arc::new(core);

        // Connect events to the core
        Self::pipe_onion_skin_renders(main_canvas.clone(), onion_skin_model.clone(), core.clone());
        Self::pipe_selected_layer_overlay(main_canvas.clone(), view_model.frame().frame.clone(), view_model.frame_update_count(), core.clone());

        // Create the controller
        let controller = CanvasController {
            ui:                 ui,
            canvases:           Arc::new(canvases),
            anim_model:         view_model.clone(),
            tool_changed:       tool_changed,
            _onion_skin_model:  onion_skin_model,

            core:               core
        };

        // Load the initial set of frame layers
        controller.core.sync(|core| {
            core.update_layers_to_frame_at_time(view_model.timeline().current_time.get());
            core.draw_frame_layers(&main_canvas);
        });

        controller
    }

    ///
    /// Creates a binding from a model to the parameters of the onion skin renderer function
    ///
    fn onion_skin_binding(view_model: &FloModel<Anim>) -> BindRef<(Color, Color, Vec<(OnionSkinTime, Arc<Vec<Draw>>)>)> {
        let onion_skin_model    = view_model.onion_skin();
        let past_color          = onion_skin_model.past_color.clone();
        let future_color        = onion_skin_model.future_color.clone();
        let onion_skins         = onion_skin_model.onion_skins.clone();

        BindRef::from(computed(move || {
            (past_color.get(), future_color.get(), onion_skins.get())
        }))
    }

    ///
    /// Updates the rendering in the core whenever the onion skins change
    ///
    fn pipe_onion_skin_renders(canvas: Resource<BindingCanvas>, binding: BindRef<(Color, Color, Vec<(OnionSkinTime, Arc<Vec<Draw>>)>)>, core: Arc<Desync<CanvasCore<Anim>>>) {
        let onion_skin_stream   = follow(binding);
        let renderer            = OnionSkinRenderer::new();

        pipe_in(core, onion_skin_stream, move |core, (past_color, future_color, onion_skins)| {
            renderer.render(&*canvas, &mut core.renderer, onion_skins, past_color, future_color);
            Box::pin(future::ready(()))
        })
    }

    ///
    /// Draws the overlay for the currently selected layer whenever it changes
    ///
    fn pipe_selected_layer_overlay(canvas: Resource<BindingCanvas>, frame: BindRef<Option<Arc<dyn Frame>>>, frame_edit_counter: BindRef<u64>, core: Arc<Desync<CanvasCore<Anim>>>) {
        // Create a stream of frame updates
        let selected_frame          = computed(move || {
            // Fetch the current edit
            let frame       = frame.get();
            let edit_count  = frame_edit_counter.get();

            // TODO: always updating every time the edit count is changed could get expensive if there are a lot of overlays
            // (right now just expecting a couple of animation elements)
            (frame, edit_count)
        });
        let selected_frame_stream   = follow(selected_frame);

        // Update the overlay whenever the frame changes
        pipe_in(core, selected_frame_stream, move |core, (frame, _edit_count)| {
            core.redraw_overlays(&canvas, &frame);

            Box::pin(future::ready(()))
        });
    }

    ///
    /// Creates the ui for the canvas controller
    ///
    fn ui(main_canvas: Resource<BindingCanvas>, size: BindRef<(f64, f64)>) -> BindRef<Control> {
        let ui = computed(move || {
            let main_canvas     = main_canvas.clone();
            let size            = size.get();
            let (width, height) = size;
            let (width, height) = (width as f32, height as f32);

            Control::scrolling_container()
                .with(Bounds::fill_all())
                .with(Scroll::MinimumContentSize(width, height))
                .with(vec![
                    Control::canvas()
                        .with(main_canvas)
                        .with(Bounds::fill_all())
                        .with(Hint::FastDrawing)
                        .with((
                            (ActionTrigger::Paint(PaintDevice::Pen),                        PAINT_ACTION),
                            (ActionTrigger::Paint(PaintDevice::Other),                      PAINT_ACTION),
                            (ActionTrigger::Paint(PaintDevice::Eraser),                     PAINT_ACTION),
                            (ActionTrigger::Paint(PaintDevice::Mouse(MouseButton::Left)),   PAINT_ACTION)
                        ))
                ])
        });

        BindRef::from(ui)
    }

    ///
    /// Create the canvas for this controller
    ///
    fn create_main_canvas(resources: &ResourceManager<BindingCanvas>) -> Resource<BindingCanvas> {
        let canvas          = resources.register(BindingCanvas::new());
        resources.assign_name(&canvas, MAIN_CANVAS);

        canvas
    }

    ///
    /// Performs a series of painting actions on the canvas
    ///
    fn paint(&self, device: &PaintDevice, actions: &Vec<Painting>) {
        let device = *device;

        // Update the paint device in the tool model if we're starting a new paint action
        if actions.len() > 0 && actions[0].action == PaintAction::Start {
            self.anim_model.tools().current_pointer.set((device, actions[0].pointer_id));
        }

        // Fetch the canvas we're going to draw to
        let canvas = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();

        // Convert the actions into tool inputs
        let tool_inputs = actions.iter()
            .map(|painting| ToolInput::Paint(painting.clone()));

        // Send to the canvas tools object
        self.core.sync(move |core| {
            let mut extra_inputs = vec![];

            // If the paint device has changed, then send a tool input indicating that that has occurred
            if Some(device) != core.last_paint_device {
                core.last_paint_device = Some(device);
                extra_inputs.push(ToolInput::PaintDevice(device));
            }

            // Amend the inputs
            let tool_inputs = extra_inputs.into_iter().chain(tool_inputs);

            // Send the inputs
            core.canvas_tools.send_input(&canvas, &mut core.renderer, tool_inputs)
        });
    }

    ///
    /// Runs the canvas update loop
    ///
    async fn run_canvas(core: Arc<Desync<CanvasCore<Anim>>>) {
        // The 'retired' edits are edits that have been written out to the animation
        let mut retired_edits = core.sync(|core| core.model.retired_edits()).ready_chunks(100);

        while let Some(next_edit) = retired_edits.next().await {
            // Send edits to the core
            // When this wakes up the main loop, this generates a tick, which will finish updating the canvas if the edits invalidate it
            core.future_desync(move |core| {
                async move {
                    for edit in next_edit {
                        core.process_edits(&*edit.committed_edits());
                    }
                }.boxed()
            }).await.ok();
        }
    }
}

impl<Anim: Animation+EditableAnimation+'static> Controller for CanvasController<Anim> {
    fn ui(&self) -> BindRef<Control> {
        BindRef::clone(&self.ui)
    }

    fn tick(&self) {
        let canvas = self.canvases.get_named_resource(MAIN_CANVAS).unwrap();

        if self.core.sync(|core| core.pending_finish_action) {
            self.core.desync(|core| { core.pending_finish_action = false; });
            self.anim_model.perform_edits(vec![AnimationEdit::Undo(UndoEdit::FinishAction)]);
        }

        // Ensure that the active tool is up to date
        if *self.tool_changed.lock().unwrap() {
            // Tool has changed: need to call refresh()
            self.core.sync(|core| {
                // Tool has no longer changed
                *self.tool_changed.lock().unwrap() = false;

                // Refresh the tool (this will update any overlays, for example)
                core.canvas_tools.refresh_tool(&*canvas, &mut core.renderer);
            });
        } else {
            // Process any pending actions for the current tool
            self.core.sync(|core| {
                core.canvas_tools.poll_for_pending_actions(&*canvas, &mut core.renderer)
            });
        }

        // Check that the frame time hasn't changed and the frame has not been invalidated since it was last drawn
        self.core.sync(|core| core.update_canvas(&canvas));
    }

    fn action(&self, action_id: &str, action_parameter: &ActionParameter) {
        use self::ActionParameter::*;

        match (action_id, action_parameter) {
            (PAINT_ACTION, &Paint(ref device, ref painting))    => self.paint(device, painting),
            _                                                   => ()
        };
    }

    fn get_canvas_resources(&self) -> Option<Arc<ResourceManager<BindingCanvas>>> {
        Some(self.canvases.clone())
    }

    fn runtime(&self) -> Option<BoxFuture<'static, ()>> {
        Some(Self::run_canvas(Arc::clone(&self.core)).boxed())
    }
}

impl<Anim: 'static+Animation+EditableAnimation> CanvasCore<Anim> {
    ///
    /// Computes the frames for all the layers in the animation
    ///
    fn update_layers_to_frame_at_time(&mut self, time: Duration) {
        // Retrieve the layers from the animation
        let layers              = self.model.frame().layers.get();
        let timeline_layers     = self.model.timeline().layers.get();
        let invalidate_count    = self.model.timeline().canvas_invalidation_count.get();

        // Update the time set
        self.current_time               = time;
        self.current_invalidation_count = invalidate_count;

        // Clear any existing canvases
        self.renderer.clear();

        // Load the frames into the renderer
        for layer_frame in layers {
            let timeline_layer = timeline_layers.iter().filter(|layer| layer.id == layer_frame.layer_id).nth(0);
            let timeline_layer = if let Some(timeline_layer) = timeline_layer { timeline_layer } else { continue; };

            self.renderer.load_frame(&layer_frame, timeline_layer);
        }
    }

    ///
    /// Draws the current set of frame layers
    ///
    fn draw_frame_layers(&mut self, canvas: &Resource<BindingCanvas>) {
        let size    = self.model.size();

        // Draw the active set of layers
        self.renderer.draw_frame_layers(&**canvas, size);
        self.renderer.draw_overlays(&**canvas);
    }

    ///
    /// Redraws the overlays on the canvas
    ///
    fn redraw_overlays(&mut self, canvas: &Resource<BindingCanvas>, frame: &Option<Arc<dyn Frame>>) {
        if let Some(frame) = frame {
            // Draw a new overlay for the frame
            let mut overlay_drawing     = vec![Draw::ClearLayer];
            frame.render_overlay(&mut overlay_drawing);

            // Draw as the layer overlay
            self.renderer.overlay(&canvas, OVERLAY_ELEMENTS, overlay_drawing);
        } else {
            // Clear the overlay
            self.renderer.overlay(&canvas, OVERLAY_ELEMENTS, vec![Draw::ClearLayer]);
        }
    }

    ///
    /// Redraws any invalidated region of the canvas using the invalidation list in the model
    ///
    fn update_canvas(&mut self, canvas: &Resource<BindingCanvas>) {
        // Take the current list of invalidations from the canvas (we'll process these now)
        let mut invalidations           = self.model.timeline().take_canvas_invalidations();

        // Add a 'whole canvas' invalidation if the time has updated
        let displayed_time              = self.current_time;
        let target_time                 = self.model.timeline().current_time.get();

        if displayed_time != target_time {
            self.current_time = target_time;
            invalidations.push(CanvasInvalidation::WholeCanvas);
        }

        // Work out the invalid layers/whole canvas invalidation from the list of instructions
        let mut whole_canvas_invalid    = false;
        let mut invalid_layers          = HashSet::new();
        for invalidation in invalidations {
            match invalidation {
                CanvasInvalidation::WholeCanvas     => { whole_canvas_invalid = true; }
                CanvasInvalidation::Layer(layer_id) => { invalid_layers.insert(layer_id); }
            }
        }

        // Redraw the appropriate parts of the canvas
        if whole_canvas_invalid {
            // Refresh the entire canvas
            self.update_layers_to_frame_at_time(target_time);
            self.draw_frame_layers(canvas);
        } else if invalid_layers.len() > 0 {
            self.renderer.clear_annotation(&*canvas);

            // Refresh individual layers
            let timeline_layers         = self.model.timeline().layers.get();
            let invalid_layer_models    = self.model.frame().layers.get()
                .into_iter()
                .filter(|frame_layer| invalid_layers.contains(&frame_layer.layer_id));

            for invalid_layer in invalid_layer_models {
                let layer_id        = invalid_layer.layer_id;

                let timeline_layer  = timeline_layers.iter().filter(|layer| layer.id == layer_id).nth(0);
                let timeline_layer  = if let Some(timeline_layer) = timeline_layer { timeline_layer } else { continue; };

                self.renderer.load_frame(&invalid_layer, timeline_layer);
                self.renderer.redraw_layer(layer_id, &*canvas);
            }

            // Update any out of date layer alphas
            self.renderer.update_layer_alphas(&*canvas);
        } else {
            // Just update the alphas            
            self.renderer.update_layer_alphas(&*canvas);
        }
    }

    ///
    /// Updates the canvas according to a set of edits that have been committed to it
    ///
    fn process_edits(&mut self, edits: &Vec<AnimationEdit>) {
        // The layers are used to determine which layers are affected by an element edit
        let layers = self.model.frame().layers.get();

        // Everything that happens in a single tick should show up as a single undo action
        if !edits.is_empty() && edits != &vec![AnimationEdit::Undo(UndoEdit::FinishAction)] {
           self.pending_finish_action = true;
        }

        for edit in edits.iter() {
            match edit {
                AnimationEdit::Motion(_element, _motion)        => { /* Motions are deprecated */ }
                AnimationEdit::SetSize(_w, _h)                  => { self.model.timeline().invalidate_canvas(); }
                AnimationEdit::SetFrameLength(_length)          => { self.model.timeline().invalidate_canvas(); }
                AnimationEdit::SetLength(_length)               => { }
                AnimationEdit::AddNewLayer(_layer_id)           => { }
                AnimationEdit::RemoveLayer(_layer_id)           => { }

                AnimationEdit::Layer(layer_id, edit)            => {
                    // Determine if this edit should refresh the layer
                    let update_layer = match edit {
                        LayerEdit::Paint(_, _)                                  => { false /* Tool is responsible for the update, for efficiency when we're just overlaying a brush stroke */ }
                        LayerEdit::Path(_, _)                                   => { false /* Tool is also responsible here */ }
                        LayerEdit::CreateAnimation(_, _, _)                     => { true }
                        LayerEdit::CreateElement(_, _, _)                       => { true }
                        LayerEdit::CreateElementUnattachedToFrame(_, _, _)      => { false }
                        LayerEdit::Cut { path: _, when: _, inside_group: _ }    => { true }
                        LayerEdit::AddKeyFrame(_)                               => { true }
                        LayerEdit::RemoveKeyFrame(_)                            => { true },
                        LayerEdit::SetName(_)                                   => { false },
                        LayerEdit::SetOrdering(_)                               => { self.model.timeline().invalidate_canvas(); false /* ... but whole canvas update */ },
                        LayerEdit::SetAlpha(_)                                  => { true },
                    };

                    // Force the layer to update if necessary
                    if update_layer {
                        self.model.timeline().invalidate_canvas_layer(*layer_id);
                    }
                },

                AnimationEdit::Element(elements, edit)          => {
                    // Determine if we should update the layer(s) that these elements are on
                    let update_layer = match edit {
                        ElementEdit::AttachTo(_)                    => false,
                        ElementEdit::AddAttachment(_)               => false,
                        ElementEdit::RemoveAttachment(_)            => false,
                        ElementEdit::SetControlPoints(_, _)         => true,
                        ElementEdit::SetPath(_)                     => true,
                        ElementEdit::Order(_)                       => true,
                        ElementEdit::Delete                         => true,
                        ElementEdit::Group(_, _)                    => true,
                        ElementEdit::Ungroup                        => false,
                        ElementEdit::CollideWithExistingElements    => false,
                        ElementEdit::ConvertToPath                  => false,
                        ElementEdit::Transform(_)                   => true,
                        ElementEdit::SetAnimationDescription(_)     => true,
                        ElementEdit::SetAnimationBaseType(_)        => true,
                        ElementEdit::AddAnimationEffect(_)          => true,
                        ElementEdit::ReplaceAnimationEffect(_, _)   => true,
                    };

                    // Update all of the layers for all of the elements if needed
                    if update_layer {
                        for layer in layers.iter() {
                            let frame               = layer.frame.get();
                            let frame               = frame.as_ref();
                            let update_this_layer   = elements.iter().any(|element_id| 
                                frame.map(|frame| frame.element_with_id(*element_id).is_some()).unwrap_or(false));

                            if update_this_layer {
                                self.model.timeline().invalidate_canvas_layer(layer.layer_id);
                            }
                        }
                    }
                },

                AnimationEdit::Undo(_)                          => { },
            }
        }
    }
}

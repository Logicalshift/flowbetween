use super::overlay_layers::*;
use super::canvas_renderer::*;
use super::super::tools::*;
use super::super::model::*;

use flo_ui::*;
use flo_canvas::*;
use flo_stream::*;
use flo_binding::*;
use flo_animation::*;
use flo_animation::brushes::*;

use std::iter;
use std::sync::*;
use std::time::Duration;

///
/// Converts tool actions into actions for a canvas
///
pub struct CanvasTools<Anim: Animation+EditableAnimation> {
    /// The animation that actions should be committed to
    animation: Arc<FloModel<Anim>>,

    /// The edit sink for the animation
    edit_sink: Publisher<Arc<Vec<AnimationEdit>>>,

    /// The effective tool for the animation
    effective_tool: BindRef<Option<Arc<FloTool<Anim>>>>,

    /// Whether or not we should create a keyframe if one doesn't already exist before committing an action
    create_keyframe: BindRef<bool>,

    /// Whether or not we should combine the new element with existing elements after the next commit
    /// TODO: this doesn't need to be a stateful setting, it's like this to minimize the set of changes required to move the
    /// code to do this out of the brush preview implementation
    combine_after_commit: bool,

    /// The time where editing is taking place
    current_time: BindRef<Duration>,

    /// The active brush preview, if there is one
    preview: Option<BrushPreview>,

    /// The brush preview layer
    preview_layer: Option<u64>,

    /// The name of the active tool
    active_tool: Option<Arc<FloTool<Anim>>>,

    /// The brush definition that has been set
    brush_definition: (BrushDefinition, BrushDrawingStyle),

    /// The brush properties that have been set
    brush_properties: BrushProperties,

    /// Runs commands for the active tool
    tool_runner: ToolRunner<Anim>
}

impl<Anim: 'static+Animation+EditableAnimation> CanvasTools<Anim> {
    ///
    /// Creates a new canvas tools structure
    ///
    pub fn from_model(view_model: &FloModel<Anim>) -> CanvasTools<Anim> {
        let animation       = Arc::new(view_model.clone());
        let effective_tool  = BindRef::from(view_model.tools().effective_tool.clone());
        let current_time    = BindRef::from(view_model.timeline().current_time.clone());
        let tool_runner     = ToolRunner::new(view_model);
        let create_keyframe = BindRef::from(view_model.frame().create_keyframe_on_draw.clone());
        let edit_sink       = animation.edit();

        CanvasTools {
            animation:              animation,
            edit_sink:              edit_sink,
            effective_tool:         effective_tool,
            create_keyframe:        create_keyframe,
            current_time:           current_time,
            preview:                None,
            preview_layer:          None,
            combine_after_commit:   false,
            active_tool:            None,
            tool_runner:            tool_runner,
            brush_definition:       (BrushDefinition::Simple, BrushDrawingStyle::Draw),
            brush_properties:       BrushProperties::new()
        }
    }

    ///
    /// Deselects the active tool
    ///
    fn deselect_active_tool(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer) {
        // Send and process the deselect actions to whatever tool is active
        let deselect_actions = self.tool_runner.actions_for_input(iter::once(ToolInput::Deselect));
        self.process_actions(canvas, renderer, deselect_actions.into_iter());

        // Active tool becomes 'None' once the deselect actions are processed (so there can be no further feedback to it)
        self.active_tool = None;
    }

    ///
    /// Sends the selection action to the current tool
    ///
    fn select_active_tool(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer) {
        // Send and process the select action to whatever tool is active
        let select_actions = self.tool_runner.actions_for_input(iter::once(ToolInput::Select));
        self.process_actions(canvas, renderer, select_actions.into_iter());
    }

    ///
    /// If the effective tool is different, changes the tool that's being used by the tool runner
    ///
    pub fn refresh_tool(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer) {
        let effective_tool = self.effective_tool.get();

        // If the tool is different...
        if self.active_tool != effective_tool {
            // ... check that a tool is actually selected
            if let Some(effective_tool) = effective_tool {
                // Deselect the current tool
                self.deselect_active_tool(canvas, renderer);

                // Select a new tool
                self.active_tool = Some(Arc::clone(&effective_tool));

                // Fetch the model for this tool
                let tool_model = self.animation.tools().model_for_tool(&*effective_tool, Arc::clone(&self.animation));

                // Load into the tool runner
                self.tool_runner.set_tool(&effective_tool, &*tool_model);

                // Clear the tool overlay
                renderer.overlay(canvas, OVERLAY_TOOL, vec![Draw::ClearCanvas]);

                // Process the 'select' action for the new tool
                self.select_active_tool(canvas, renderer);
            }
        }
    }

    ///
    /// Polls for any pending actions (for example, because the model was updated)
    ///
    pub fn poll_for_pending_actions(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer) {
        // Get any pending tool actions
        let actions = self.tool_runner.model_actions();

        // Process any tool data actions
        let mut remaining_actions = vec![];
        for action in actions {
            match action {
                ToolAction::Data(new_data)  => self.tool_runner.set_tool_data(new_data),
                not_tool_data               => remaining_actions.push(not_tool_data)
            }
        }

        // Pass the remaining actions to process_actions
        if remaining_actions.len() > 0 {
            self.process_actions(canvas, renderer, remaining_actions.into_iter());
        }
    }

    ///
    /// Sends input to the current tool
    ///
    pub fn send_input<InputIter: Iterator<Item=ToolInput<GenericToolData>>>(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer, input: InputIter) {
        // Ensure that the tool is ready to run
        self.refresh_tool(canvas, renderer);

        // Send the input to the tool to get the actions
        let actions = self.tool_runner.actions_for_input(input);

        // Process the actions
        self.process_actions(canvas, renderer, actions);
    }

    ///
    /// Processes a set of actions with whatever tool is selected, rendering them if necessary
    ///
    /// Call `refresh_tool` before calling this to make sure that the effective tool
    /// is active.
    ///
    pub fn process_actions<ActionIter: Iterator<Item=ToolAction<GenericToolData>>>(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer, actions: ActionIter) {
        // Process the actions in sequence
        let mut animation_edits = vec![];

        for action in actions {
            match action {
                ToolAction::Data(data)              => self.tool_runner.set_tool_data(data),
                ToolAction::Edit(edit)              => animation_edits.push(edit),
                ToolAction::BrushPreview(preview)   => self.process_brush_preview(canvas, renderer, preview),
                ToolAction::Overlay(overlay)        => self.process_overlay(canvas, renderer, overlay),
                ToolAction::Select(element)         => self.animation.selection().select(element),
                ToolAction::ClearSelection          => self.animation.selection().clear_selection(),
                ToolAction::InvalidateFrame         => self.animation.timeline().invalidate_canvas()
            }
        }

        // Commit any animation edits that the tool produced
        if animation_edits.len() > 0 {
            self.animation.perform_edits(animation_edits);
        }

        // If there's a brush preview, draw it as the renderer annotation
        if let Some(preview) = self.preview.as_ref() {
            if let Some(preview_layer) = self.preview_layer {
                let need_brush = self.need_brush_definition(preview_layer, renderer);
                let need_props = self.need_brush_properties(preview_layer, renderer);

                renderer.annotate_layer(canvas, preview_layer, |gc| preview.draw_current_brush_stroke(gc, need_brush, need_props));
            }
        }
    }

    ///
    /// True if we need to update the brush definition before drawing
    ///
    fn need_brush_definition(&self, layer_id: u64, renderer: &CanvasRenderer) -> bool {
        let (brush, _properties) = renderer.get_layer_brush(layer_id);

        brush.as_ref() != Some(&self.brush_definition)
    }

    ///
    /// True if we need to update the brush properties before drawing
    ///
    fn need_brush_properties(&self, layer_id: u64, renderer: &CanvasRenderer) -> bool {
        let (_brush, properties) = renderer.get_layer_brush(layer_id);

        properties.as_ref() != Some(&self.brush_properties)
    }

    ///
    /// Processes a brush preview action
    ///
    fn process_brush_preview(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer, preview: BrushPreviewAction) {
        match preview {
            BrushPreviewAction::Clear                           => {
                let mut preview = BrushPreview::new();
                preview.set_brush_properties(&self.brush_properties);
                preview.select_brush(&self.brush_definition.0, self.brush_definition.1);

                self.preview                = Some(preview);
                self.combine_after_commit   = false;
            },

            BrushPreviewAction::UnsetProperties                 => { self.preview_layer.map(|layer_id| renderer.set_layer_brush(layer_id, None, None)); }
            BrushPreviewAction::Layer(layer_id)                 => { self.preview_layer = Some(layer_id); },
            BrushPreviewAction::BrushDefinition(defn, style)    => { self.brush_definition = (defn.clone(), style); self.preview.as_mut().map(move |preview| preview.select_brush(&defn, style)); },
            BrushPreviewAction::BrushProperties(props)          => { self.brush_properties = props; self.preview.as_mut().map(move |preview| preview.set_brush_properties(&props)); },
            BrushPreviewAction::AddPoint(point)                 => { self.preview.as_mut().map(move |preview| preview.continue_brush_stroke(point)); },
            BrushPreviewAction::Commit                          => { self.commit_brush_preview(canvas, renderer) },
            BrushPreviewAction::CommitAsPath                    => { self.commit_brush_preview_as_path(canvas, renderer) }
            BrushPreviewAction::CombineCollidingElements        => { self.combine_colliding_elements() }
        }
    }

    ///
    /// Process an overlay action
    ///
    fn process_overlay(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer, overlay: OverlayAction) {
        // Overlay 0 is used for tool overlays

        match overlay {
            OverlayAction::Clear            => renderer.overlay(canvas, OVERLAY_TOOL, vec![ Draw::ClearCanvas ]),
            OverlayAction::Draw(drawing)    => renderer.overlay(canvas, OVERLAY_TOOL, drawing)
        }
    }

    ///
    /// Returns true if the current time in the preview layer isn't currently on a keyframe
    ///
    fn need_new_keyframe(&self) -> bool {
        if let Some(preview_layer) = self.preview_layer {
            // Look for a frame around the current time
            let current_time    = self.current_time.get();
            let one_ms          = Duration::from_millis(1);
            let earliest_time   = if current_time > one_ms { current_time - one_ms } else { Duration::from_millis(0) };
            let latest_time     = current_time + one_ms;

            let layer           = self.animation.get_layer_with_id(preview_layer);
            let keyframes       = layer.map(|layer| layer.get_key_frames_during_time(earliest_time..latest_time).collect::<Vec<_>>());

            // If there is no keyframe at this time, then we need to create a new keyframe here
            keyframes.map(|keyframes| keyframes.len() == 0).unwrap_or(false)
        } else {
            // No preview layer (so we can't create a keyframe)
            false
        }
    }

    ///
    /// Creates a new keyframe if there is no current keyframe and the 'create keyframe on draw' option is set.
    ///
    /// Returns true if the keyframe was created
    ///
    fn create_new_keyframe_if_required(&mut self) -> bool {
        if let Some(preview_layer) = self.preview_layer {
            if self.create_keyframe.get() {
                if self.need_new_keyframe() {
                    let current_time    = self.current_time.get();

                    // Create a keyframe at this time
                    self.animation.perform_edits(vec![AnimationEdit::Layer(preview_layer, LayerEdit::AddKeyFrame(current_time))]);

                    // Canvas should be invalidated
                    self.animation.timeline().invalidate_canvas();
                    self.animation.timeline().update_keyframe_bindings();

                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    ///
    /// Commits the current brush preview to the animation
    ///
    fn commit_brush_preview(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer) {
        // We take the preview here (so there's no preview after this)
        if let (Some(mut preview), Some(preview_layer)) = (self.preview.take(), self.preview_layer) {
            let mut need_brush  = self.need_brush_definition(preview_layer, renderer);
            let mut need_props  = self.need_brush_properties(preview_layer, renderer);

            let current_time    = self.current_time.get();

            // Create a new keyframe for this brush stroke if necessary
            if self.create_new_keyframe_if_required() {
                // Will need to define the brush & properties
                need_brush = true;
                need_props = true;
            }

            // Commit the brush stroke to the renderer
            renderer.commit_to_layer(canvas, preview_layer, |gc| preview.draw_current_brush_stroke(gc, need_brush, need_props));

            // Commit the preview to the animation
            let elements = preview.commit_to_animation(need_brush, need_props, current_time, preview_layer, &*self.animation);
            if self.combine_after_commit {
                self.animation.perform_edits(vec![AnimationEdit::Element(elements, ElementEdit::CollideWithExistingElements)]);
            }

            // Update the properties in the renderer if they've changed
            if need_brush || need_props {
                renderer.set_layer_brush(preview_layer, Some(self.brush_definition.clone()), Some(self.brush_properties.clone()));
            }
        }
    }

    ///
    /// Turns the brush preview into a path and commits it to the animation
    ///
    fn commit_brush_preview_as_path(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer) {
        // Take the brush preview and commit
        if let (Some(mut preview), Some(preview_layer)) = (self.preview.take(), self.preview_layer) {
            let mut need_brush  = self.need_brush_definition(preview_layer, renderer);
            let mut need_props  = self.need_brush_properties(preview_layer, renderer);
            let current_time    = self.current_time.get();

            // Create a new keyframe for this brush stroke if necessary
            if self.create_new_keyframe_if_required() {
                // Will need to define the brush & properties
                need_brush = true;
                need_props = true;
            }

            // Commit the brush stroke to the renderer
            renderer.commit_to_layer(canvas, preview_layer, |gc| preview.draw_current_brush_stroke(gc, need_brush, need_props));

            // Commit the preview to the animation
            let elements        = preview.commit_to_animation(need_brush, need_props, current_time, preview_layer, &*self.animation);
            let mut path_edits  = vec![AnimationEdit::Element(elements.clone(), ElementEdit::ConvertToPath)];

            if self.combine_after_commit {
                path_edits.insert(0, AnimationEdit::Element(elements, ElementEdit::CollideWithExistingElements));
            }

            self.animation.perform_edits(path_edits);
        }
    }

    ///
    /// Causes the brush preview to combine any elements that are overlapping (so we combine them into one path)
    ///
    fn combine_colliding_elements(&mut self) {
        self.combine_after_commit = true;
    }
}

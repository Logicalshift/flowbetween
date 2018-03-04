use super::canvas_renderer::*;
use super::super::tools::*;
use super::super::model::*;

use ui::*;
use binding::*;
use animation::*;
use animation::brushes::*;

use std::iter;
use std::sync::*;
use std::time::Duration;

///
/// Converts tool actions into actions for a canvas
/// 
pub struct CanvasTools<Anim: Animation> {
    /// The animation that actions should be committed to
    animation: Arc<FloModel<Anim>>,

    /// The effective tool for the animation
    effective_tool: BindRef<Option<Arc<FloTool<Anim>>>>,

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

impl<Anim: 'static+Animation> CanvasTools<Anim> {
    ///
    /// Creates a new canvas tools structure
    /// 
    pub fn from_model(view_model: &FloModel<Anim>) -> CanvasTools<Anim> {
        let animation       = Arc::new(view_model.clone());
        let effective_tool  = BindRef::from(view_model.tools().effective_tool.clone());
        let current_time    = BindRef::from(view_model.timeline().current_time.clone());
        let tool_runner     = ToolRunner::new(view_model);

        CanvasTools {
            animation:          animation,
            effective_tool:     effective_tool,
            current_time:       current_time,
            preview:            None,
            preview_layer:      None,
            active_tool:        None,
            tool_runner:        tool_runner,
            brush_definition:   (BrushDefinition::Simple, BrushDrawingStyle::Draw),
            brush_properties:   BrushProperties::new()
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
                let tool_model = self.animation.tools().model_for_tool(&*effective_tool);

                // Load into the tool runner
                self.tool_runner.set_tool(&effective_tool, &*tool_model);

                // Process the 'select' action for the new tool
                self.select_active_tool(canvas, renderer);
            }
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
    /// Processes a set of actions, rendering them if necessary
    /// 
    pub fn process_actions<ActionIter: Iterator<Item=ToolAction<GenericToolData>>>(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer, actions: ActionIter) {
        // Ensure that the tool is ready to run
        self.refresh_tool(canvas, renderer);

        // Process the actions in sequence
        let mut animation_edits = vec![];

        for action in actions {
            match action {
                ToolAction::Data(data)              => self.tool_runner.set_tool_data(data),
                ToolAction::Edit(edit)              => animation_edits.push(edit),
                ToolAction::BrushPreview(preview)   => self.process_brush_preview(canvas, renderer, preview),
                ToolAction::Overlay(_)              => unimplemented!()
            }
        }

        // Commit any animation edits that the tool produced
        if animation_edits.len() > 0 {
            let mut editor = self.animation.edit();
            editor.set_pending(&animation_edits);
            editor.commit_pending();
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

                self.preview = Some(preview);
            },

            BrushPreviewAction::Layer(layer_id)                 => { self.preview_layer = Some(layer_id); },
            BrushPreviewAction::BrushDefinition(defn, style)    => { self.brush_definition = (defn.clone(), style); self.preview.as_mut().map(move |preview| preview.select_brush(&defn, style)); },
            BrushPreviewAction::BrushProperties(props)          => { self.brush_properties = props; self.preview.as_mut().map(move |preview| preview.set_brush_properties(&props)); },
            BrushPreviewAction::AddPoint(point)                 => { self.preview.as_mut().map(move |preview| preview.continue_brush_stroke(point)); },
            BrushPreviewAction::Commit                          => { self.commit_brush_preview(canvas, renderer) }
        }
    }

    ///
    /// Commits the current brush preview to the animation
    /// 
    fn commit_brush_preview(&mut self, canvas: &BindingCanvas, renderer: &mut CanvasRenderer) {
        // We take the preview here (so there's no preview after this)
        if let Some(mut preview) = self.preview.take() {
            // The preview layer is left behind: the next brush stroke will be on the same layer if a new one is not specified
            if let Some(preview_layer) = self.preview_layer {
                let need_brush = self.need_brush_definition(preview_layer, renderer);
                let need_props = self.need_brush_properties(preview_layer, renderer);

                // Commit the brush stroke to the renderer
                renderer.commit_to_layer(canvas, preview_layer, |gc| preview.draw_current_brush_stroke(gc, need_brush, need_props));

                // Commit the preview to the animation
                preview.commit_to_animation(need_brush, need_props, self.current_time.get(), preview_layer, &*self.animation);

                // Update the properties in the renderer if they've changed
                if need_brush || need_props {
                    renderer.set_layer_brush(preview_layer, Some(self.brush_definition.clone()), Some(self.brush_properties.clone()));
                }
            }
        }
    }
}

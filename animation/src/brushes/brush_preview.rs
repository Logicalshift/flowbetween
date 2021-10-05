use super::*;
use super::super::traits::*;

use flo_canvas::*;

use std::mem;
use std::time::Duration;

///
/// The brush preview structure is used to create and render a brush preview
///
pub struct BrushPreview {
    current_brush:          Arc<dyn Brush>,
    brush_properties:       BrushProperties,
    raw_points:             Vec<RawPoint>,
    brush_points:           Option<Arc<Vec<BrushPoint>>>,
    combined_element:       Option<Vector>
}

impl BrushPreview {
    pub fn new() -> Self {
        Self {
            current_brush:      create_brush_from_definition(&BrushDefinition::Simple, BrushDrawingStyle::Draw),
            brush_properties:   BrushProperties::new(),
            raw_points:         vec![],
            brush_points:       None,
            combined_element:   None
        }
    }

    ///
    /// Chooses the brush that we should draw with
    ///
    pub fn select_brush(&mut self, brush: &BrushDefinition, drawing_style: BrushDrawingStyle) {
        // TODO: store brush definitions in the animation
        self.current_brush = create_brush_from_definition(brush, drawing_style);
    }

    ///
    /// Sets the properties for the current brush
    ///
    /// (Always sets the 'changed' flag)
    ///
    pub fn set_brush_properties(&mut self, properties: &BrushProperties) {
        self.brush_properties = *properties;
    }

    ///
    /// Continues the current brush stroke
    ///
    pub fn continue_brush_stroke(&mut self, point: RawPoint) {
        // Add points to the active brush stroke
        self.brush_points = None;
        self.raw_points.push(point);
    }

    ///
    /// Clears the preview
    ///
    pub fn cancel_brush_stroke(&mut self) {
        self.raw_points         = vec![];
        self.brush_points       = None;
        self.combined_element   = None;
    }

    ///
    /// Creates the definition element for the current brush stroke
    ///
    pub fn brush_definition_element(&self) -> BrushDefinitionElement {
        let (defn, drawing_style) = self.current_brush.to_definition();
        BrushDefinitionElement::new(ElementId::Unassigned, defn, drawing_style)
    }

    ///
    /// Creates the properties element for the current brush stroke
    ///
    pub fn brush_properties_element(&self) -> BrushPropertiesElement {
        BrushPropertiesElement::new(ElementId::Unassigned, self.brush_properties)
    }

    ///
    /// Returns the set of brush points for this element
    ///
    fn brush_points(&self) -> Arc<Vec<BrushPoint>> {
        if let Some(brush_points) = &self.brush_points {
            Arc::clone(brush_points)
        } else {
            Arc::new(self.current_brush.brush_points_for_raw_points(&self.raw_points))
        }
    }

    ///
    /// Sets the brush points to use in the preview for this brush stroke
    ///
    pub fn set_brush_points(&mut self, brush_points: Arc<Vec<BrushPoint>>) {
        self.brush_points = Some(brush_points);
    }

    ///
    /// Creates the brush element for the current brush stroke
    ///
    pub fn brush_element(&self) -> BrushElement {
        let brush_points = self.brush_points();

        BrushElement::new(ElementId::Unassigned, brush_points)
    }

    ///
    /// Writes the brush definition to the current layer
    ///
    pub fn commit_brush_definition(&self, when: Duration, layer_id: u64, animation: &dyn EditableAnimation) {
        use LayerEdit::*;
        use PaintEdit::*;

        let (defn, drawing_style) = self.current_brush.to_definition();

        animation.perform_edits(vec![
            AnimationEdit::Layer(layer_id, Paint(when, SelectBrush(ElementId::Unassigned, defn, drawing_style)))
        ]);
    }

    ///
    /// Writes the brush properties to the current layer
    ///
    pub fn commit_brush_properties(&self, when: Duration, layer_id: u64, animation: &dyn EditableAnimation) {
        use LayerEdit::*;
        use PaintEdit::*;

        animation.perform_edits(vec![
            AnimationEdit::Layer(layer_id, Paint(when, BrushProperties(ElementId::Unassigned, self.brush_properties.clone())))
        ]);
    }

    ///
    /// Updates the brush settings for the current layer
    ///
    pub fn commit_brush_settings(&self, when: Duration, layer_id: u64, animation: &dyn EditableAnimation) {
        self.commit_brush_definition(when, layer_id, animation);
        self.commit_brush_properties(when, layer_id, animation);
    }

    ///
    /// Draws this preview brush stroke to the specified graphics object
    ///
    pub fn draw_current_brush_stroke(&self, gc: &mut dyn GraphicsContext, update_brush_definition: bool, update_properties: bool) {
        if self.raw_points.len() < 2 && self.brush_points.is_none() {
            // Do nothing if there are no points in this brush preview
            return;
        }

        // Set the brush to use in the vector properties
        let mut vector_properties   = VectorProperties::default();
        vector_properties.brush     = self.current_brush.clone();

        // Render them to the canvas if they're marked as changed
        if update_brush_definition {
            self.brush_definition_element().render_static(gc, &vector_properties, Duration::from_millis(0))
        }

        // Apply brush to the vector properties
        let new_properties = self.brush_properties_element();

        // We always apply the properties so that our vector properties are accurate
        let mut vector_properties   = Arc::new(vector_properties);
        vector_properties           = new_properties.update_properties(vector_properties, Duration::from_millis(0));

        // We only render the properties if they're marked as updated
        if update_properties {
            new_properties.render_static(gc, &vector_properties, Duration::from_millis(0));
        }

        // Draw the current brush stroke
        let brush_element = self.brush_element();
        vector_properties = brush_element.update_properties(vector_properties, Duration::from_millis(0));
        brush_element.render_static(gc, &vector_properties, Duration::from_millis(0));
    }

    ///
    /// Commits this preview to an animation, returning the element IDs that were generated
    ///
    pub fn commit_to_animation(&mut self, update_brush_definition: bool, update_properties: bool, when: Duration, layer_id: u64, animation: &dyn EditableAnimation) -> Vec<ElementId> {
        use LayerEdit::*;
        use PaintEdit::*;

        if self.raw_points.len() < 2 {
            // Do nothing if there are no points in this brush preview
            return vec![];
        }

        let mut actions = vec![];

        // Select the brush
        if update_brush_definition {
            let (defn, drawing_style) = self.current_brush.to_definition();

            actions.push(Paint(when, SelectBrush(ElementId::Unassigned, defn, drawing_style)));
        }

        // Select the properties
        if update_properties {
            actions.push(Paint(when, BrushProperties(ElementId::Unassigned, self.brush_properties.clone())));
        }

        // Perform the brush stroke (and clear out the points)
        let element_id  = animation.assign_element_id();
        let mut points  = vec![];
        let _combined   = self.combined_element.take();
        mem::swap(&mut self.raw_points, &mut points);
        actions.push(Paint(when, BrushStroke(element_id, Arc::new(points))));

        // Perform the edit
        let actions         = actions.into_iter().map(|action| AnimationEdit::Layer(layer_id, action));
        animation.perform_edits(actions.collect());

        vec![element_id]
    }
}

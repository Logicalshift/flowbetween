use super::*;
use super::super::traits::*;

use flo_canvas::*;

use futures::executor;
use std::mem;
use std::time::Duration;

///
/// The brush preview structure is used to create and render a brush preview
/// 
pub struct BrushPreview {
    current_brush:          Arc<dyn Brush>,
    brush_properties:       BrushProperties,
    points:                 Vec<RawPoint>,
}

impl BrushPreview {
    pub fn new() -> BrushPreview {
        BrushPreview {
            current_brush:      create_brush_from_definition(&BrushDefinition::Simple, BrushDrawingStyle::Draw),
            brush_properties:   BrushProperties::new(),
            points:             vec![]
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
        self.points.push(point);
    }

    ///
    /// Clears the preview
    /// 
    pub fn cancel_brush_stroke(&mut self) {
        self.points = vec![];
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
    /// Creates the brush element for the current brush stroke
    /// 
    pub fn brush_element(&self) -> BrushElement {
        let brush_points = self.current_brush.brush_points_for_raw_points(&self.points);

        BrushElement::new(ElementId::Unassigned, Arc::new(brush_points))
    }

    ///
    /// Draws this preview brush stroke to the specified graphics object
    /// 
    pub fn draw_current_brush_stroke(&self, gc: &mut dyn GraphicsPrimitives, update_brush_definition: bool, update_properties: bool) {
        if self.points.len() < 2 {
            // Do nothing if there are no points in this brush preview
            return;
        }

        // Set the brush to use in the vector properties
        let mut vector_properties   = VectorProperties::default();
        vector_properties.brush     = self.current_brush.clone();

        // Render them to the canvas if they're marked as changed
        if update_brush_definition {
            self.brush_definition_element().render(gc, &vector_properties)
        }

        // Apply brush to the vector properties
        let new_properties = self.brush_properties_element();

        // We always apply the properties so that our vector properties are accurate
        let mut vector_properties   = Arc::new(vector_properties);
        vector_properties           = new_properties.update_properties(vector_properties);

        // We only render the properties if they're marked as updated
        if update_properties {
            new_properties.render(gc, &vector_properties);
        }

        // Draw the current brush stroke
        let brush_element = self.brush_element();
        vector_properties = brush_element.update_properties(vector_properties);
        brush_element.render(gc, &vector_properties);
    }

    ///
    /// Commits this preview to an animation
    /// 
    pub fn commit_to_animation(&mut self, update_brush_definition: bool, update_properties: bool, when: Duration, layer_id: u64, animation: &dyn EditableAnimation) {
        use LayerEdit::*;
        use PaintEdit::*;

        if self.points.len() < 2 {
            // Do nothing if there are no points in this brush preview
            return;
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
        let mut points = vec![];
        mem::swap(&mut self.points, &mut points);
        actions.push(Paint(when, BrushStroke(ElementId::Unassigned, Arc::new(points))));

        // Perform the edit
        let actions         = actions.into_iter().map(|action| AnimationEdit::Layer(layer_id, action));
        let mut edit_sink   = executor::spawn(animation.edit());

        edit_sink.wait_send(actions.collect()).ok();
    }

    ///
    /// Commits a brush preview to the animation as a path element
    ///
    pub fn commit_to_animation_as_path(&mut self, when: Duration, layer_id: u64, animation: &dyn EditableAnimation) {
        use PathEdit::*;

        let mut actions = vec![];

        if self.points.len() < 2 {
            // Do nothing if there are no points in this brush preview
            return;
        }

        // Path properties
        let (defn, drawing_style) = self.current_brush.to_definition();
        actions.push(LayerEdit::Path(when, SelectBrush(ElementId::Unassigned, defn, drawing_style)));
        actions.push(LayerEdit::Path(when, BrushProperties(ElementId::Unassigned, self.brush_properties.clone())));

        // Path itself
        let brush_points    = self.current_brush.brush_points_for_raw_points(&self.points);
        let path            = vec![Path::from_drawing(self.current_brush.render_brush(&self.brush_properties, &brush_points))];
        let path            = path.into_iter().map(|path| path.elements().collect::<Vec<_>>()).flatten();
        actions.push(LayerEdit::Path(when, CreatePath(ElementId::Unassigned, Arc::new(path.collect()))));

        // Perform the edit
        let actions         = actions.into_iter().map(|action| AnimationEdit::Layer(layer_id, action));
        let mut edit_sink   = executor::spawn(animation.edit());

        edit_sink.wait_send(actions.collect()).ok();
    }
}

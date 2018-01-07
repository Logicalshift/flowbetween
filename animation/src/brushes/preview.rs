use super::*;
use super::super::traits::*;

use canvas::*;

use std::mem;
use std::time::Duration;

///
/// The brush preview structure is used to create and render a brush preview
/// 
pub struct BrushPreview {
    current_brush:          Arc<Brush>,
    brush_properties:       BrushProperties,
    points:                 Vec<RawPoint>,

    brush_changed:          bool,
    properties_changed:     bool,
    finished:               bool
}

impl BrushPreview {
    pub fn new() -> BrushPreview {
        BrushPreview {
            current_brush:      create_brush_from_definition(&BrushDefinition::Simple, BrushDrawingStyle::Draw),
            brush_properties:   BrushProperties::new(),
            points:             vec![],
            brush_changed:      false,
            properties_changed: false,
            finished:           true
        }
    }

    ///
    /// Chooses the brush that we should draw with
    /// 
    pub fn select_brush(&mut self, brush: &BrushDefinition, drawing_style: BrushDrawingStyle) {
        // TODO: store brush definitions in the animation
        self.current_brush = create_brush_from_definition(brush, drawing_style);
        self.brush_changed = true;
    }

    ///
    /// Sets the properties for the current brush
    /// 
    /// (Always sets the 'changed' flag)
    /// 
    pub fn set_brush_properties(&mut self, properties: &BrushProperties) {
        self.brush_properties = *properties;
        self.properties_changed = true;
    }

    ///
    /// Updates the properties for the current brush
    /// 
    /// (Won't mark them as changed if they're the same as the current properties)
    /// 
    pub fn update_brush_properties(&mut self, properties: &BrushProperties) {
        if properties != &self.brush_properties {
            self.brush_properties = *properties;
            self.properties_changed = true;
        }
    }

    ///
    /// Starts a new brush stroke
    /// 
    pub fn start_brush_stroke(&mut self, initial_pos: RawPoint) {
        self.finished = false;
        self.points = vec![initial_pos];
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
        self.finished = true;
        self.points = vec![];
    }

    ///
    /// Creates the definition element for the current brush stroke
    /// 
    pub fn brush_definition_element(&self) -> BrushDefinitionElement {
        let (defn, drawing_style) = self.current_brush.to_definition();
        BrushDefinitionElement::new(defn, drawing_style)
    }

    ///
    /// Creates the properties element for the current brush stroke
    /// 
    pub fn brush_properties_element(&self) -> BrushPropertiesElement {
        BrushPropertiesElement::new(self.brush_properties)
    }

    ///
    /// Creates the brush element for the current brush stroke
    /// 
    pub fn brush_element(&self) -> BrushElement {
        let brush_points = self.current_brush.brush_points_for_raw_points(&self.points);

        BrushElement::new(Arc::new(brush_points))
    }

    ///
    /// Draws this preview brush stroke to the specified graphics object
    /// 
    pub fn draw_current_brush_stroke(&self, gc: &mut GraphicsPrimitives) {
        let mut vector_properties = VectorProperties::default();

        // Set the brush to use in the vector properties
        vector_properties.brush = self.current_brush.clone();

        // Render them to the canvas if they're marked as changed
        if self.brush_changed {
            self.brush_definition_element().render(gc, &vector_properties)
        }

        // Apply brush to the vector properties
        let new_properties = self.brush_properties_element();
        new_properties.update_properties(&mut vector_properties);

        // Render them to the canvas if they're marked as changed
        if self.properties_changed {
            new_properties.render(gc, &vector_properties);
        }

        // Draw the current brush stroke
        let brush_element = self.brush_element();
        brush_element.update_properties(&mut vector_properties);
        brush_element.render(gc, &vector_properties);
    }

    ///
    /// Commits this preview to an animation
    /// 
    pub fn commit_to_animation(&mut self, when: Duration, layer_id: u64, animation: &Animation) {
        use LayerEdit::*;
        use PaintEdit::*;

        let mut actions = vec![];

        self.finished = true;

        // Select the brush
        if self.brush_changed {
            let (defn, drawing_style) = self.current_brush.to_definition();

            actions.push(Paint(when, SelectBrush(defn, drawing_style)));
            self.brush_changed = false
        }

        // Select the properties
        if self.properties_changed {
            actions.push(Paint(when, BrushProperties(self.brush_properties.clone())));
            self.properties_changed = false;
        }

        // Perform the brush stroke (and clear out the points)
        let mut points = vec![];
        mem::swap(&mut self.points, &mut points);
        actions.push(Paint(when, BrushStroke(Arc::new(points))));

        // Perform the edit
        let mut edit = animation.edit_layer(layer_id);

        edit.set_pending(&actions);
        edit.commit_pending();
    }

    ///
    /// True if this brush stroke has been cancelled or committed
    /// 
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}
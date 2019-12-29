use super::*;
use super::super::traits::*;

use flo_canvas::*;
use flo_stream::*;
use flo_curves::bezier::path::*;

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
    combined_element:       Option<Vector>
}

impl BrushPreview {
    pub fn new() -> Self {
        Self {
            current_brush:      create_brush_from_definition(&BrushDefinition::Simple, BrushDrawingStyle::Draw),
            brush_properties:   BrushProperties::new(),
            points:             vec![],
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
        self.points.push(point);
    }

    ///
    /// Clears the preview
    ///
    pub fn cancel_brush_stroke(&mut self) {
        self.points             = vec![];
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
            self.brush_definition_element().render(gc, &vector_properties, Duration::from_millis(0))
        }

        // Apply brush to the vector properties
        let new_properties = self.brush_properties_element();

        // We always apply the properties so that our vector properties are accurate
        let mut vector_properties   = Arc::new(vector_properties);
        vector_properties           = new_properties.update_properties(vector_properties);

        // We only render the properties if they're marked as updated
        if update_properties {
            new_properties.render(gc, &vector_properties, Duration::from_millis(0));
        }

        // Draw the current brush stroke
        let brush_element = self.brush_element();
        vector_properties = brush_element.update_properties(vector_properties);
        brush_element.render(gc, &vector_properties, Duration::from_millis(0));
    }

    ///
    /// Discovers all of the elements in the frame along with their properties
    ///
    fn frame_elements_with_properties(&self, frame: Arc<dyn Frame>) -> Vec<(Vector, Arc<VectorProperties>)> {
        // Start with the default properties
        let mut current_properties  = Arc::new(VectorProperties::default());
        let mut result              = vec![];

        // If this is a vector frame, apply the properties from each element
        if let Some(vector_elements) = frame.vector_elements() {
            for elem in vector_elements {
                // Update the properties for this element
                current_properties = frame.apply_properties_for_element(&elem, current_properties);

                // Add to the result
                result.push((elem, Arc::clone(&current_properties)));
            }
        }

        result
    }

    ///
    /// Attempts to combine the preview brush stroke with existing elements in the frame
    ///
    pub fn collide_with_existing_elements(&mut self, frame: Arc<dyn Frame>) {
        // We need to know the properties of all of the elements in the current frame (we need to work backwards to generate the grouped element)
        let elements_with_properties        = self.frame_elements_with_properties(frame);
        let brush_points                    = Arc::new(self.current_brush.brush_points_for_raw_points(&self.points));

        // Nothing to do if there are no properties
        if elements_with_properties.len() == 0 {
            return;
        }

        // The vector properties of the brush will be the last element properties with the brush properties added in
        let mut new_properties              = (*elements_with_properties.last().unwrap().1).clone();
        new_properties.brush                = Arc::clone(&self.current_brush);
        new_properties.brush_properties     = self.brush_properties.clone();

        // Attempt to combine the current brush stroke with them
        let mut combined_element            = None;
        for (element, properties) in elements_with_properties.iter().rev() {
            use self::CombineResult::*;

            combined_element = match self.current_brush.combine_with(&element, Arc::clone(&brush_points), &new_properties, &*properties, combined_element.clone()) {
                NewElement(new_combined)    => { Some(new_combined) },
                NoOverlap                   => { continue; },               // Might be able to combine with an element further down
                CannotCombineAndOverlaps    => { break; },                  // Not quite right: we can combine with any element that's not obscured by an existing element (we can skip over overlapping elements we can't combine with)
                UnableToCombineFurther      => { break; }                   // Always stop here
            }
        }

        self.combined_element = combined_element;
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
        let mut points  = vec![];
        let _combined   = self.combined_element.take();
        mem::swap(&mut self.points, &mut points);
        actions.push(Paint(when, BrushStroke(ElementId::Unassigned, Arc::new(points))));

        // Perform the edit
        let actions         = actions.into_iter().map(|action| AnimationEdit::Layer(layer_id, action));
        let mut edit_sink   = animation.edit();

        executor::block_on(async { edit_sink.publish(actions.collect()).await });
    }

    ///
    /// Generates a list of edits to perform before committing a combined path
    ///
    fn edits_before_commiting_combined_path<'a>(&'a self, delete_elements_instead_of_detatching: bool) -> impl 'a+Iterator<Item=AnimationEdit> {
        // If we're grouping a set of elements, then the existing elements need to either be deleted or detached
        let elements_to_remove = self.combined_element
            .iter()
            .flat_map(|element| {
                match element {
                    Vector::Group(group) => Some(group.elements().filter(|element| element.id() != ElementId::Unassigned).map(|element| element.id()).collect()),

                    _ => None
                }
            });

        let edit = if delete_elements_instead_of_detatching { ElementEdit::Delete } else { ElementEdit::DetachFromFrame };

        elements_to_remove
            .into_iter()
            .map(move |element_ids| AnimationEdit::Element(element_ids, edit.clone()))
    }

    ///
    /// Commits a brush preview to the animation as a path element
    ///
    pub fn commit_to_animation_as_path(&mut self, when: Duration, layer_id: u64, animation: &dyn EditableAnimation) {
        use PathEdit::*;

        let mut actions = vec![];

        if self.points.len() < 2 {
            // Do nothing if there are no points in this brush preview
            self.points = vec![];
            self.combined_element.take();
            return;
        }

        // Path properties (TODO: don't add path properties if they're already set correctly, maybe?)
        let (defn, drawing_style) = self.current_brush.to_definition();
        actions.push(AnimationEdit::Layer(layer_id, LayerEdit::Path(when, SelectBrush(ElementId::Unassigned, defn, drawing_style))));
        actions.push(AnimationEdit::Layer(layer_id, LayerEdit::Path(when, BrushProperties(ElementId::Unassigned, self.brush_properties.clone()))));

        // Path itself
        let mut vector_properties           = VectorProperties::default();
        vector_properties.brush             = self.current_brush.clone();
        vector_properties.brush_properties  = vector_properties.brush_properties;

        let brush_points                    = self.current_brush.brush_points_for_raw_points(&self.points);
        let brush_element                   = BrushElement::new(ElementId::Unassigned, Arc::new(brush_points));

        let path = if let Some(combined) = self.combined_element.as_ref() {
            let path                        = combined.to_path(&vector_properties);
            let path                        = path.unwrap_or(vec![]).into_iter();
            let path                        = path.filter(|path| path.elements().count() > 2);
            let path                        = path.map(|path| path.elements().collect::<Vec<_>>()).flatten();
            path.collect::<Vec<_>>()
        } else {
            let path                        = brush_element.to_path(&vector_properties);
            let path                        = path.into_iter().flatten();
            let path                        = path.filter(|path| path.elements().count() > 2);
            let path                        = path.map(|path| path_remove_interior_points::<_, Path>(&vec![path], 0.01)).flatten();
            let path                        = path.map(|path| path.elements().collect::<Vec<_>>()).flatten();
            path.collect::<Vec<_>>()
        };

        if path.len() <= 0 {
            return;
        }

        actions.extend(self.edits_before_commiting_combined_path(true));
        actions.push(AnimationEdit::Layer(layer_id, LayerEdit::Path(when, CreatePath(ElementId::Unassigned, Arc::new(path)))));

        // Perform the edit
        let mut edit_sink   = animation.edit();

        executor::block_on(async {
            edit_sink.publish(actions).await
        });

        self.points             = vec![];
        self.combined_element   = None;
    }
}

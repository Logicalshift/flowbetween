use super::*;
use super::db_enum::*;
use super::flo_store::*;
use super::flo_query::*;

use canvas::*;

use std::time::Duration;

///
/// Represents a frame calculated from a vector layer
/// 
pub struct VectorFrame {
    /// The time of the keyframe
    keyframe_time: Duration,

    /// Time from the start of the keyframe that this frame is at
    keyframe_offset: Duration,

    /// The elements in this frame
    elements: Vec<Vector>
}

impl VectorFrame {
    ///
    /// Decodes the brush for a particular entry
    /// 
    fn brush_definition_for_entry<TFile: FloFile>(db: &mut TFile, entry: VectorElementEntry) -> Result<BrushDefinitionElement> {
        // Try to load the brush with the ID
        let brush: Result<(BrushDefinition, DrawingStyleType)> = entry.brush
            .map(|(brush_id, drawing_style)| Ok((AnimationDbCore::get_brush_definition(db, brush_id)?, drawing_style)))
            .unwrap_or_else(|| Ok((BrushDefinition::Ink(InkDefinition::default()), DrawingStyleType::Draw)));

        // Generate a brush element from it
        let (brush, drawing_style) = brush?;

        let drawing_style = match drawing_style {
            DrawingStyleType::Draw  => BrushDrawingStyle::Draw,
            DrawingStyleType::Erase => BrushDrawingStyle::Erase
        };

        Ok(BrushDefinitionElement::new(entry.assigned_id, brush, drawing_style))
    }

    ///
    /// Decodes the brush properties for a particular entry
    /// 
    fn properties_for_entry<TFile: FloFile>(db: &mut TFile, entry: VectorElementEntry) -> Result<BrushPropertiesElement> {
        // Decode the brush properties
        let brush_properties = entry.brush_properties_id
            .map(|brush_properties_id| AnimationDbCore::get_brush_properties(db, brush_properties_id))
            .unwrap_or_else(|| Ok(BrushProperties::new()));

        // Generate the element
        Ok(BrushPropertiesElement::new(entry.assigned_id, brush_properties?))
    }

    ///
    /// Decodes a brush stroke element for a particular element
    /// 
    fn brush_stroke_for_entry<TFile: FloFile>(db: &mut TFile, entry: VectorElementEntry) -> Result<BrushElement> {
        let points = db.query_vector_element_brush_points(entry.element_id)?;
        Ok(BrushElement::new(entry.assigned_id, Arc::new(points)))
    }

    ///
    /// Tries to turn a vector element entry into a Vector object
    /// 
    fn vector_for_entry<TFile: FloFile>(db: &mut TFile, entry: VectorElementEntry) -> Result<Vector> {
        match entry.element_type {
            VectorElementType::BrushDefinition => Ok(Vector::BrushDefinition(Self::brush_definition_for_entry(db, entry)?)),
            VectorElementType::BrushProperties => Ok(Vector::BrushProperties(Self::properties_for_entry(db, entry)?)),
            VectorElementType::BrushStroke     => Ok(Vector::BrushStroke(Self::brush_stroke_for_entry(db, entry)?))
        }
    }

    ///
    /// Creates a vector frame by querying the file for the frame at the specified time
    /// 
    pub fn frame_at_time<TFile: FloFile>(db: &mut TFile, layer_id: i64, when: Duration) -> Result<VectorFrame> {
        // Fetch the keyframe times
        let (keyframe_id, keyframe_time)    = db.query_nearest_key_frame(layer_id, when)?;
        let keyframe_offset                 = when - keyframe_time;

        // Read the elements for this layer
        let vector_entries = db.query_vector_keyframe_elements_before(keyframe_id, keyframe_offset)?;

        // Process the elements
        let mut elements = vec![];
        for entry in vector_entries {
            elements.push(Self::vector_for_entry(db, entry)?);
        }

        // Can create the frame now
        Ok(VectorFrame {
            keyframe_time:      keyframe_time,
            keyframe_offset:    keyframe_offset,
            elements:           elements
        })
    }
}

impl Frame for VectorFrame {
    ///
    /// Time index of this frame
    /// 
    fn time_index(&self) -> Duration {
        self.keyframe_time + self.keyframe_offset
    }

    ///
    /// Renders this frame to a particular graphics context
    ///
    fn render_to(&self, gc: &mut GraphicsPrimitives) {
        let mut properties = VectorProperties::default();

        self.elements.iter().for_each(move |element| {
            // Properties always update regardless of the time they're at (so the display is consistent)
            element.update_properties(&mut properties);
            element.render(gc, &properties);
        })
    }

    ///
    /// Attempts to retrieve the vector elements associated with this frame, if there are any
    /// 
    fn vector_elements<'a>(&'a self) -> Option<Box<'a+Iterator<Item=Vector>>> {
        Some(Box::new(self.elements.iter().cloned()))
    }

    ///
    /// Finds the brush that will be active after this frame has rendered
    /// 
    fn active_brush(&self) -> Option<(BrushDefinition, BrushDrawingStyle)> {
        let mut properties          = VectorProperties::default();
        let mut changes_definition  = false;

        self.elements.iter()
            .for_each(|element| {
                // There are only active brush properties if they've been explicitly set
                match element {
                    &Vector::BrushDefinition(_) => changes_definition = true,
                    _ => ()
                };

                // Update our properties element from this element
                element.update_properties(&mut properties);
            });

        if changes_definition {
            Some(properties.brush.to_definition())
        } else {
            None
        }
    }

    ///
    /// Finds the brush properties that will be active after this frame has rendered
    /// 
    fn active_brush_properties(&self) -> Option<BrushProperties> {
        let mut properties          = VectorProperties::default();
        let mut changes_properties  = false;

        self.elements.iter()
            .for_each(|element| {
                // There are only active brush properties if they've been explicitly set
                match element {
                    &Vector::BrushProperties(_) => changes_properties = true,
                    _ => ()
                };

                // Update our properties element from this element
                element.update_properties(&mut properties);
            });

        // Return the properties that we found if there were any updates
        if changes_properties {
            Some(properties.brush_properties)
        } else {
            None
        }
    }
}

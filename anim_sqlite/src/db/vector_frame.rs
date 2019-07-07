use super::*;
use super::db_enum::*;
use super::flo_store::*;
use super::flo_query::*;

use flo_canvas::*;

use std::time::Duration;
use std::collections::HashMap;

///
/// Represents a frame calculated from a vector layer
/// 
pub struct VectorFrame {
    /// The time of the keyframe
    keyframe_time: Duration,

    /// Time from the start of the keyframe that this frame is at
    keyframe_offset: Duration,

    /// The top-level elements in this frame, in order
    elements: Vec<Vector>,

    /// Maps element IDs to their assigned IDs
    element_id_for_assigned_id: HashMap<ElementId, i64>,

    /// Hashmap of element IDs to vector frame element for all elements in this frame
    all_elements: HashMap<i64, Vector>,

    /// List of the attachments for each element in the frame
    attachments: HashMap<ElementId, Vec<(ElementId, VectorType)>>
}

impl VectorFrame {
    ///
    /// Decodes the brush for a particular entry
    /// 
    fn brush_definition_for_entry<TFile: FloFile+Send>(db: &mut TFile, entry: VectorElementEntry) -> Result<BrushDefinitionElement> {
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
    fn properties_for_entry<TFile: FloFile+Send>(db: &mut TFile, entry: VectorElementEntry) -> Result<BrushPropertiesElement> {
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
    fn brush_stroke_for_entry<TFile: FloFile+Send>(db: &mut TFile, entry: VectorElementEntry) -> Result<BrushElement> {
        let points = db.query_vector_element_brush_points(entry.element_id)?;
        Ok(BrushElement::new(entry.assigned_id, Arc::new(points)))
    }

    ///
    /// Returns the path element associated with a particular entry
    ///
    fn path_for_entry<TFile: FloFile+Send>(db: &mut TFile, entry: VectorElementEntry) -> Result<PathElement> {
        let path_entry                  = db.query_path_element(entry.element_id)?;
        let path_entry                  = path_entry.unwrap();

        let brush_id                    = path_entry.brush_id;
        let properties_id               = path_entry.brush_properties_id;
        let brush_entry                 = db.query_vector_element(brush_id)?;
        let brush_properties_entry      = db.query_vector_element(properties_id)?;

        let brush                       = Self::brush_definition_for_entry(db, brush_entry)?;
        let brush_properties            = Self::properties_for_entry(db, brush_properties_entry)?;
        let path_components             = db.query_path_components(path_entry.path_id)?;
        let path                        = Path::from(path_components);

        Ok(PathElement::new(entry.assigned_id, path, Arc::new(brush), Arc::new(brush_properties)))
    }

    ///
    /// Returns the motion element associated with a particular entry
    ///
    fn motion_for_entry<TFile: FloFile+Send>(db: &mut TFile, entry: VectorElementEntry) -> Result<MotionElement> {
        // The motion ID is the assigned ID for the entry
        // (This is a quirk due to the original design; a better approach would be to use the main element ID with the current database structure)
        let motion_id       = entry.assigned_id.id().unwrap();

        // Load the data fr the motion
        let motion_entry    = db.query_motion(motion_id)?.unwrap();
        let motion          = AnimationDb::motion_for_entry(db, motion_id, motion_entry)?;

        // Generate the motion element
        Ok(MotionElement::new(entry.assigned_id, motion))
    }

    ///
    /// Tries to turn a vector element entry into a Vector object
    /// 
    fn vector_for_entry<TFile: FloFile+Send>(db: &mut TFile, entry: VectorElementEntry) -> Result<Vector> {
        match entry.element_type {
            VectorElementType::BrushDefinition      => Ok(Vector::BrushDefinition(Self::brush_definition_for_entry(db, entry)?)),
            VectorElementType::BrushProperties      => Ok(Vector::BrushProperties(Self::properties_for_entry(db, entry)?)),
            VectorElementType::BrushStroke          => Ok(Vector::BrushStroke(Self::brush_stroke_for_entry(db, entry)?)),
            VectorElementType::Path                 => Ok(Vector::Path(Self::path_for_entry(db, entry)?)),
            VectorElementType::Motion               => Ok(Vector::Motion(Self::motion_for_entry(db, entry)?))
        }
    }

    ///
    /// Creates a vector frame by querying the file for the frame at the specified time
    /// 
    pub fn frame_at_time<TFile: FloFile+Send>(db: &mut TFile, layer_id: i64, when: Duration) -> Result<VectorFrame> {
        // Fetch the keyframe times
        if let Some((keyframe_id, keyframe_time))   = db.query_nearest_key_frame(layer_id, when)? {
            let keyframe_offset = when - keyframe_time;

            // Read the elements for this layer
            let vector_entries  = db.query_vector_keyframe_elements_and_attachments_before(keyframe_id, keyframe_offset)?;

            // Process the elements
            let mut root_elements   = vec![];
            let mut element_ids     = HashMap::new();
            let mut all_elements    = HashMap::new();
            let mut attachments     = HashMap::new();

            for entry in vector_entries {
                let raw_element_id = entry.vector.element_id;

                // Attachment elements might be returned multiple times from the file, so we don't bother re-creating them if they do
                if all_elements.contains_key(&raw_element_id) {
                    // Just add as an attached element if we've already generated the entry in all_elements
                    if let (ElementId::Assigned(_), ElementId::Assigned(element_id)) = (entry.attached_to_assigned_id, entry.vector.assigned_id) {
                        // Is an attached element
                        attachments.entry(entry.attached_to_assigned_id)
                            .or_insert_with(|| vec![])
                            .push((ElementId::Assigned(element_id), entry.vector.element_type.into()));
                    }
                    continue;
                }

                // Fetch the vector element from the key frame
                let mut vector          = Self::vector_for_entry(db, entry.vector)?;

                // Generate the element entry for this element
                let element_id          = vector.id();

                // Finished generating the vector for this element
                if entry.attached_to_element.is_some() {
                    // Is an attached element
                    if entry.attached_to_assigned_id.is_assigned() {
                        attachments.entry(entry.attached_to_assigned_id)
                            .or_insert_with(|| vec![])
                            .push((element_id, VectorType::from(&vector)));
                    }
                } else {
                    // Is a root element
                    root_elements.push(vector.clone());
                }

                // Add this as one of the 'all elements' hash table
                element_ids.insert(element_id, raw_element_id);
                all_elements.insert(raw_element_id, vector);
            }

            // Can create the frame now
            Ok(VectorFrame {
                keyframe_time:              keyframe_time,
                keyframe_offset:            keyframe_offset,
                elements:                   root_elements,
                all_elements:               all_elements,
                attachments:                attachments,
                element_id_for_assigned_id: element_ids
            })
        } else {
            // No keyframe
            Ok(VectorFrame {
                keyframe_time:              Duration::from_micros(0),
                keyframe_offset:            when,
                elements:                   vec![],
                all_elements:               HashMap::new(),
                attachments:                HashMap::new(),
                element_id_for_assigned_id: HashMap::new()
            })
        }
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
    fn render_to(&self, gc: &mut dyn GraphicsPrimitives) {
        let mut properties          = Arc::new(VectorProperties::default());
        let mut active_attachments  = vec![];
        let when                    = self.time_index();

        self.elements.iter().for_each(move |element| {
            // Fetch the attachment IDs
            let element_attachments = self.attached_elements(element.id()).into_iter().map(|(id, _type)| id).collect::<Vec<_>>();

            // Update the properties based on the attachments, if the attachments are different
            if active_attachments != element_attachments {
                // These attachments are active now
                active_attachments = element_attachments;

                // Apply them to generate the properties for the following elements
                properties = Arc::new(VectorProperties::default());
                for element_id in active_attachments.iter() {
                    if let Some(attach_element) = self.element_with_id(element_id.clone()) {
                        properties = attach_element.update_properties(Arc::clone(&properties));
                        properties.render(gc, attach_element, when);
                    }
                }
            }

            // Render the element via the properties
            // TODO: avoid the clone here somehow
            properties.render(gc, element.clone(), when);
        })
    }

    ///
    /// Applies all of the properties for the specified element (including those added by attached elements)
    ///
    fn apply_properties_for_element(&self, element: &Vector, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        let mut properties = properties;

        // Get the attachments for this element
        let element_attachments = self.attached_elements(element.id()).into_iter().map(|(id, _type)| id).collect::<Vec<_>>();

        // Apply them to the properties
        for element_id in element_attachments.iter() {
            if let Some(attach_element) = self.element_with_id(element_id.clone()) {
                properties = attach_element.update_properties(properties);
            }
        }

        // Apply the properties added by the main element
        properties = element.update_properties(properties);

        properties
    }

    ///
    /// Attempts to retrieve the vector elements associated with this frame, if there are any
    /// 
    fn vector_elements<'a>(&'a self) -> Option<Box<dyn 'a+Iterator<Item=Vector>>> {
        Some(Box::new(self.elements.iter().cloned()))
    }

    ///
    /// Searches for an element with the specified ID and returns it if found within this frame
    /// 
    fn element_with_id(&self, id: ElementId) -> Option<Vector> {
        self.element_id_for_assigned_id.get(&id)
            .and_then(|id| self.all_elements.get(id))
            .cloned()
    }

    ///
    /// Retrieves the IDs and types of the elements attached to the element with a particular ID
    /// 
    /// (Element data can be retrieved via element_with_id)
    ///
    fn attached_elements(&self, id: ElementId) -> Vec<(ElementId, VectorType)> {
        self.attachments.get(&id)
            .cloned()
            .unwrap_or_else(|| vec![])
    }
}

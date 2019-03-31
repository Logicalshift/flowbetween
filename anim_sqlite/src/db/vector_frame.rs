use super::*;
use super::db_enum::*;
use super::flo_store::*;
use super::flo_query::*;

use flo_canvas::*;

use std::rc::*;
use std::time::Duration;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

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
    /// Tries to turn a vector element entry into a Vector object
    /// 
    fn vector_for_entry<TFile: FloFile+Send>(db: &mut TFile, entry: VectorElementEntry) -> Result<Vector> {
        match entry.element_type {
            VectorElementType::BrushDefinition      => Ok(Vector::BrushDefinition(Self::brush_definition_for_entry(db, entry)?)),
            VectorElementType::BrushProperties      => Ok(Vector::BrushProperties(Self::properties_for_entry(db, entry)?)),
            VectorElementType::BrushStroke          => Ok(Vector::BrushStroke(Self::brush_stroke_for_entry(db, entry)?)),
            VectorElementType::Path                 => Ok(Vector::Path(Self::path_for_entry(db, entry)?)),
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

            // If there are any motions for the elements, we cache them here
            let mut motions     = HashMap::new();

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

                // Fetch the motions that are attached to this element and apply them
                let mut element_motions = vec![];
                if let ElementId::Assigned(id) = vector.id() {
                    // Get the motions attached to this element
                    let motion_ids = db.query_motion_ids_for_element(id)?;

                    // Collect them into a list
                    for motion_id in motion_ids {
                        // Fetch the motion (from the cache or the database)
                        let motion = match motions.entry(motion_id) {
                            Entry::Occupied(cached_motion)  => Rc::clone(cached_motion.get()),

                            Entry::Vacant(vacant_motion)    => {
                                // Motion is not cached: fetch from the database
                                let motion_entry = db.query_motion(motion_id)?;

                                // Convert to an actual motion
                                let motion = motion_entry
                                    .map(|motion_entry| AnimationDb::motion_for_entry(db, motion_id, motion_entry))
                                    .unwrap_or(Ok(Motion::None))?;
                                let motion = Rc::new(motion);
                                
                                // Store in the entry
                                vacant_motion.insert(Rc::clone(&motion));
                                motion
                            }
                        };
                        
                        // Store this as a motion applying to this element
                        element_motions.push(motion);
                    }
                }

                // Apply each motion in turn to this element
                for motion in element_motions {
                    vector = vector.motion_transform(&*motion, when);
                }


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

        self.elements.iter().for_each(move |element| {
            // Fetch the attachment IDs
            let element_attachments = self.attached_elements(element.id()).into_iter().map(|(id, _type)| id).collect::<Vec<_>>();

            // Update the properties based on the attachments, if the attachments are different
            if active_attachments != element_attachments {
                // These attachments are active now
                active_attachments = element_attachments;

                // Apply them to the current set of properties
                for element_id in active_attachments.iter() {
                    if let Some(attach_element) = self.element_with_id(element_id.clone()) {
                        properties = attach_element.update_properties(Arc::clone(&properties));
                        attach_element.render(gc, &properties);
                    }
                }
            }

            // Properties always update regardless of the time they're at (so the display is consistent)
            properties = element.update_properties(Arc::clone(&properties));
            element.render(gc, &properties);
        })
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

    ///
    /// Finds the brush that will be active after this frame has rendered
    /// 
    fn active_brush(&self) -> Option<(BrushDefinition, BrushDrawingStyle)> {
        let mut properties          = Arc::new(VectorProperties::default());
        let mut changes_definition  = false;

        self.elements.iter()
            .for_each(|element| {
                // There are only active brush properties if they've been explicitly set
                match element {
                    &Vector::BrushDefinition(_) => changes_definition = true,
                    _ => ()
                };

                // Update our properties element from this element
                properties = element.update_properties(Arc::clone(&properties));
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
        let mut properties          = Arc::new(VectorProperties::default());
        let mut changes_properties  = false;

        self.elements.iter()
            .for_each(|element| {
                // There are only active brush properties if they've been explicitly set
                match element {
                    &Vector::BrushProperties(_) => changes_properties = true,
                    _ => ()
                };

                // Update our properties element from this element
                properties = element.update_properties(Arc::clone(&properties));
            });

        // Return the properties that we found if there were any updates
        if changes_properties {
            Some(properties.brush_properties)
        } else {
            None
        }
    }
}

use super::db_enum::*;
use super::flo_store::*;

use animation::*;
use animation::brushes::*;

use rusqlite::*;
use std::sync::*;
use std::time::Duration;
use std::collections::HashMap;

///
/// Core data structure used by the animation database
/// 
pub struct AnimationDbCore<TFile: FloFile+Send> {
    /// The database connection
    pub db: TFile,

    /// If there has been a failure with the database, this is it. No future operations 
    /// will work while there's an error that hasn't been cleared
    pub failure: Option<Error>,

    /// Maps layers to the brush that's active
    pub active_brush_for_layer: HashMap<i64, (Duration, Arc<Brush>)>,

    /// Maps the assigned layer IDs to their equivalent real IDs
    pub layer_id_for_assigned_id: HashMap<u64, i64>,

    /// The next element ID that will be assigned
    pub next_element_id: i64
}

impl<TFile: FloFile+Send> AnimationDbCore<TFile> {
    ///
    /// Assigns the next element ID and returns it
    ///
    fn next_element_id(&mut self) -> i64 {
        let result      = self.next_element_id;
        self.next_element_id += 1;
        result
    }

    ///
    /// Assigns an element ID to an animation edit
    ///
    fn assign_element_id(&mut self, edit: AnimationEdit) -> AnimationEdit {
        use self::AnimationEdit::*;
        use self::LayerEdit::*;
        use self::PaintEdit::*;

        match edit {
            Layer(layer_id, Paint(when, BrushProperties(ElementId::Unassigned, props))) =>
                Layer(layer_id, Paint(when, BrushProperties(ElementId::Assigned(self.next_element_id()), props))),

            Layer(layer_id, Paint(when, SelectBrush(ElementId::Unassigned, defn, drawing_style))) =>
                Layer(layer_id, Paint(when, SelectBrush(ElementId::Assigned(self.next_element_id()), defn, drawing_style))),

            Layer(layer_id, Paint(when, BrushStroke(ElementId::Unassigned, points))) =>
                Layer(layer_id, Paint(when, BrushStroke(ElementId::Assigned(self.next_element_id()), points))),

            other => other
        }
    }

    ///
    /// Assigns element IDs to a set of animation IDs
    ///
    pub fn assign_element_ids(&mut self, edits: Vec<AnimationEdit>) -> Vec<AnimationEdit> {
        edits.into_iter()
            .map(|edit| self.assign_element_id(edit))
            .collect()
    }

    ///
    /// Retrieves the brush that is active on the specified layer at the specified time
    ///
    pub fn get_active_brush_for_layer(&mut self, layer_id: i64, when: Duration) -> Arc<Brush> {
        // If the cached active brush is at the right time, then just use that
        if let Some((time, ref brush)) = self.active_brush_for_layer.get(&layer_id) {
            if time == &when {
                return Arc::clone(&brush);
            } else {
                unimplemented!("TODO: got a brush but for the wrong time ({:?} vs {:?})", time, when);
            }
        }

        // If the time doesn't match, or nothing is cached then we need to fetch from the database
        unimplemented!("TODO: store/fetch active brush for keyframes in the database");

        // create_brush_from_definition(&BrushDefinition::Simple, BrushDrawingStyle::Draw)
    }

    ///
    /// Creates a new vector element in an animation DB core, leaving the element ID, key frame ID and time pushed on the DB stack
    ///
    /// The element is created without its associated data.
    ///
    fn create_new_element(db: &mut TFile, layer_id: i64, when: Duration, element: &PaintEdit) -> Result<()> {
        if let ElementId::Assigned(assigned_id) = element.id() {
            db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PushNearestKeyFrame(when),
                DatabaseUpdate::PushVectorElementType(VectorElementType::from(element), when),
                DatabaseUpdate::PushElementAssignId(assigned_id)
            ])?;
        } else {
            db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PushNearestKeyFrame(when),
                DatabaseUpdate::PushVectorElementType(VectorElementType::from(element), when)
            ])?;
        }

        Ok(())
    }

    ///
    /// Writes a brush properties element to the database (popping the element ID)
    ///
    fn create_brush_properties(db: &mut TFile, properties: BrushProperties) -> Result<()> {
        AnimationDbCore::insert_brush_properties(db, &properties)?;

        // Create the element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushPropertiesElement
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush definition element to the database (popping the element ID)
    ///
    fn create_brush_definition(db: &mut TFile, definition: BrushDefinition, drawing_style: BrushDrawingStyle) -> Result<()> {
        // Create the brush definition
        AnimationDbCore::insert_brush(db, &definition)?;

        // Insert the properties for this element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushElement(DrawingStyleType::from(&drawing_style))
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush stroke to the database (popping the element ID)
    ///
    fn create_brush_stroke(&mut self, layer_id: i64, when: Duration, brush_stroke: Arc<Vec<RawPoint>>) -> Result<()> {
        // Convert the brush stroke to the brush points
        let active_brush = self.get_active_brush_for_layer(layer_id, when);
        let brush_stroke = active_brush.brush_points_for_raw_points(&*brush_stroke);

        // Store in the database
        self.db.update(vec![
            DatabaseUpdate::PopBrushPoints(Arc::new(brush_stroke))
        ])?;

        Ok(())
    }

    ///
    /// Adds a new vector element to a vector layer
    /// 
    fn paint_vector_layer(&mut self, layer_id: i64, when: Duration, new_element: PaintEdit) -> Result<()> {
        use animation::PaintEdit::*;

        // Update the state of this object based on the element
        match new_element {
            SelectBrush(_id, ref brush_definition, drawing_style)   => {
                // Cache the brush so that follow up drawing instructions don't need to
                self.active_brush_for_layer.insert(layer_id, (when, create_brush_from_definition(brush_definition, drawing_style)));
            },

            _ => ()
        }

        // Create a new element
        Self::create_new_element(&mut self.db, layer_id, when, &new_element)?;

        // Record the details of the element itself
        match new_element {
            SelectBrush(_id, brush_definition, drawing_style)   => Self::create_brush_definition(&mut self.db, brush_definition, drawing_style)?,
            BrushProperties(_id, brush_properties)              => Self::create_brush_properties(&mut self.db, brush_properties)?,
            BrushStroke(_id, brush_stroke)                      => self.create_brush_stroke(layer_id, when, brush_stroke)?,
        }

        // create_new_element pushes an element ID, a key frame ID and a time. The various element actions pop the element ID so we need to pop the frame ID and time
        self.db.update(vec![
            DatabaseUpdate::Pop,
            DatabaseUpdate::Pop
        ])?;

        Ok(())
    }

    ///
    /// Performs a layer edit to a vector layer
    /// 
    pub fn edit_vector_layer(&mut self, layer_id: i64, edit: LayerEdit) -> Result<()> {
        use self::LayerEdit::*;

        // Note that we can't access the core at this point (the database implies that the core is already in use)

        match edit {
            AddKeyFrame(when) => {
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopAddKeyFrame(when)
                ])?;
            },

            RemoveKeyFrame(when) => {
                self.db.update(vec![
                    DatabaseUpdate::PushLayerId(layer_id),
                    DatabaseUpdate::PopRemoveKeyFrame(when)
                ])?;
            },

            Paint(when, edit) => {
                self.paint_vector_layer(layer_id, when, edit)?;
            }
        }

        Ok(())
    }

    ///
    /// Performs an edit on this core
    /// 
    pub fn perform_edit(&mut self, edit: AnimationEdit) -> Result<()> {
        use self::AnimationEdit::*;

        match edit {
            SetSize(width, height) => {
                self.db.update(vec![
                    DatabaseUpdate::UpdateCanvasSize(width, height)
                ])?;
            },

            AddNewLayer(new_layer_id) => {
                // Create a layer with the new ID
                self.db.update(vec![
                    DatabaseUpdate::PushLayerType(LayerType::Vector),
                    DatabaseUpdate::PushAssignLayer(new_layer_id),
                    DatabaseUpdate::Pop
                ])?;
            },

            RemoveLayer(old_layer_id) => {
                // Delete this layer
                self.db.update(vec![
                    DatabaseUpdate::PushLayerForAssignedId(old_layer_id),
                    DatabaseUpdate::PopDeleteLayer
                ])?;
            },

            Layer(assigned_layer_id, layer_edit) => {
                // Look up the real layer ID (which is often different to the assigned ID)
                let layer_id = {
                    let db                          = &mut self.db;
                    let layer_id_for_assigned_id    = &mut self.layer_id_for_assigned_id;
                    let layer_id                    = *layer_id_for_assigned_id.entry(assigned_layer_id)
                        .or_insert_with(|| db.query_layer_id_for_assigned_id(assigned_layer_id).unwrap_or(-1));

                    layer_id
                };

                // Edit this layer
                self.edit_vector_layer(layer_id, layer_edit)?;
            },

            Element(id, when, edit) => {
                // TODO!
                // unimplemented!()
            },

            Motion(motion_id, edit) => {
                // TODO!
                // unimplemented!()
            }
        }

        Ok(())
    }
}

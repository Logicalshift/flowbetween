use super::*;
use super::db_enum::*;
use super::db_update::*;
use super::flo_store::*;

use animation::brushes::*;
use std::time::Duration;

///
/// Represents a vector layer in a SQLite database
/// 
pub struct SqliteVectorLayer<TFile: FloFile+Send> {
    /// The ID that was assigned to this layer
    assigned_id: u64,

    /// The ID of this layer
    layer_id: i64,

    /// The currently active brush for this layer (or none if we need to fetch this from the database)
    /// The active brush is the brush most recently added to the keyframe at the specified point in time
    active_brush: Option<(Duration, Arc<Brush>)>,

    /// Database core
    core: Arc<Desync<AnimationDbCore<TFile>>>
}

impl AnimationDb {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn get_layer_with_id(&self, assigned_id: u64) -> Option<SqliteVectorLayer<FloSqlite>> {
        SqliteVectorLayer::from_assigned_id(&self.core, assigned_id)
    }
}

impl<TFile: FloFile+Send+'static> SqliteVectorLayer<TFile> {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn from_assigned_id(core: &Arc<Desync<AnimationDbCore<TFile>>>, assigned_id: u64) -> Option<SqliteVectorLayer<TFile>> {
        // Query for the 'real' layer ID
        let layer = core.sync(|core| {
            // Fetch the layer data (we need the 'real' ID here)
            core.db.query_layer_id_for_assigned_id(assigned_id)
        });

        // If the layer exists, create a SqliteVectorLayer
        layer.ok()
            .map(|layer_id| {
                SqliteVectorLayer {
                    assigned_id:    assigned_id,
                    layer_id:       layer_id,
                    active_brush:   None,
                    core:           Arc::clone(core)
                }
            })
    }
}

impl<TFile: FloFile+Send+'static> SqliteVectorLayer<TFile> {
    ///
    /// Performs an async operation on the database
    /// 
    fn async<TFn: 'static+Send+Fn(&mut AnimationDbCore<TFile>) -> Result<()>>(&self, action: TFn) {
        self.core.async(move |core| {
            // Only run the function if there has been no failure
            if core.failure.is_none() {
                // Run the function and update the error status
                let result      = action(core);
                core.failure    = result.err();
            }
        })
    }
}

impl<TFile: FloFile+Send+'static> Layer for SqliteVectorLayer<TFile> {
    fn id(&self) -> u64 {
        self.assigned_id
    }

    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        vec![LayerEditType::Vector]
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame> {
        unimplemented!()
    }

    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>> {
        let keyframes = self.core.sync(|core| core.db.query_key_frame_times_for_layer_id(self.layer_id));

        // Turn into an iterator
        let keyframes = keyframes.unwrap_or_else(|_: Error| vec![]);
        let keyframes = Box::new(keyframes.into_iter());

        keyframes
    }

    fn add_key_frame(&mut self, when: Duration) {
        let layer_id = self.layer_id;

        self.async(move |core| {
            core.db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PopAddKeyFrame(when)
            ])?;

            Ok(())
        });
    }

    fn remove_key_frame(&mut self, when: Duration) {
        let layer_id = self.layer_id;

        self.async(move |core| {
            core.db.update(vec![
                DatabaseUpdate::PushLayerId(layer_id),
                DatabaseUpdate::PopRemoveKeyFrame(when)
            ])?;

            Ok(())
        });
    }

    fn as_vector_layer<'a>(&'a self) -> Option<Reader<'a, VectorLayer>> {
        let vector_layer = self as &VectorLayer;

        Some(Reader::new(vector_layer))
    }

    fn edit_vectors<'a>(&'a mut self) -> Option<Editor<'a, VectorLayer>> {
        let vector_layer = self as &mut VectorLayer;
 
        Some(Editor::new(vector_layer))
    }
}

impl<TFile: FloFile+Send> SqliteVectorLayer<TFile> {
    ///
    /// Creates a new vector element in an animation DB core, leaving the element ID, key frame ID and time pushed on the DB stack
    ///
    /// The element is created without its associated data.
    ///
    fn create_new_element(db: &mut TFile, layer_id: i64, when: Duration, element: &Vector) -> Result<()> {
        db.update(vec![
            DatabaseUpdate::PushLayerId(layer_id),
            DatabaseUpdate::PushNearestKeyFrame(when),
            DatabaseUpdate::PushVectorElementType(VectorElementType::from(element), when)
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush properties element to the database (popping the element ID)
    ///
    fn create_brush_properties(db: &mut TFile, properties: BrushPropertiesElement) -> Result<()> {
        AnimationDbCore::insert_brush_properties(db, properties.brush_properties())?;

        // Create the element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushPropertiesElement
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush definition element to the database (popping the element ID)
    ///
    fn create_brush_definition(db: &mut TFile, definition: BrushDefinitionElement) -> Result<()> {
        // Create the brush definition
        AnimationDbCore::insert_brush(db, definition.definition())?;

        // Insert the properties for this element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushElement(DrawingStyleType::from(&definition.drawing_style()))
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush stroke to the database (popping the element ID)
    ///
    fn create_brush_stroke(db: &mut TFile, brush_stroke: BrushElement) -> Result<()> {
        db.update(vec![
            DatabaseUpdate::PopBrushPoints(brush_stroke.points())
        ])?;

        Ok(())
    }
}

impl<TFile: FloFile+Send+'static> VectorLayer for SqliteVectorLayer<TFile> {
    fn add_element(&mut self, when: Duration, new_element: Vector) {
        use animation::Vector::*;

        let layer_id = self.layer_id;

        // Update the state of this object based on the element
        match new_element {
            BrushDefinition(ref brush_definition)   => {
                self.active_brush = Some((when, create_brush_from_definition(brush_definition.definition(), brush_definition.drawing_style())));
            },

            _ => ()
        }

        // Send the element to the core
        self.core.async(move |core| {
            core.edit(move |db| {
                // Create a new element
                Self::create_new_element(db, layer_id, when, &new_element)?;
        
                // Record the details of the element itself
                match new_element {
                    BrushDefinition(brush_definition)   => Self::create_brush_definition(db, brush_definition)?,
                    BrushProperties(brush_properties)   => Self::create_brush_properties(db, brush_properties)?,
                    BrushStroke(brush_stroke)           => Self::create_brush_stroke(db, brush_stroke)?,
                }

                // create_new_element pushes an element ID, a key frame ID and a time. The various element actions pop the element ID so we need to pop the frame ID and time
                db.update(vec![
                    DatabaseUpdate::Pop,
                    DatabaseUpdate::Pop
                ])?;

                Ok(())
            })
        });
    }

    fn active_brush(&self, when: Duration) -> Arc<Brush> {
        // If the cached active brush is at the right time and 
        if let Some((time, ref brush)) = self.active_brush {
            if time == when {
                return Arc::clone(&brush);
            } else {
                unimplemented!("TODO: got a brush but for the wrong time ({:?} vs {:?})", time, when);
            }
        }

        // If the time doesn't match, or nothing is cached then we need to fetch from the database
        unimplemented!("TODO: store/fetch active brush for keyframes in the database");

        // create_brush_from_definition(&BrushDefinition::Simple, BrushDrawingStyle::Draw)
    }
}

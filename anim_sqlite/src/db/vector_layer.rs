use super::*;
use super::db_enum::*;
use super::flo_store::*;
use super::vector_frame::*;

use animation::brushes::*;
use std::ops::{Range, Deref};
use std::time::Duration;

///
/// Represents a vector layer in a SQLite database
/// 
#[derive(Clone)]
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

    ///
    /// Adds a new vector element to this layer
    /// 
    fn paint(&mut self, db: &mut TFile, when: Duration, new_element: PaintEdit) {
        use animation::PaintEdit::*;

        let layer_id = self.layer_id;

        // Update the state of this object based on the element
        match new_element {
            SelectBrush(_id, ref brush_definition, _drawing_style)   => {
                self.active_brush = Some((when, create_brush_from_definition(brush_definition.definition(), brush_definition.drawing_style())));
            },

            _ => ()
        }

        // Create a new element
        Self::create_new_element(db, layer_id, when, &new_element)?;

        // Record the details of the element itself
        match new_element {
            SelectBrush(_id, brush_definition, drawing_style)   => Self::create_brush_definition(db, brush_definition, drawing_style)?,
            BrushProperties(_id, brush_properties)              => Self::create_brush_properties(db, brush_properties)?,
            BrushStroke(_id, brush_stroke)                      => Self::create_brush_stroke(db, brush_stroke)?,
        }

        // create_new_element pushes an element ID, a key frame ID and a time. The various element actions pop the element ID so we need to pop the frame ID and time
        db.update(vec![
            DatabaseUpdate::Pop,
            DatabaseUpdate::Pop
        ])?;
    }

    ///
    /// Performs a layer edit to this layer
    /// 
    pub fn edit(&mut self, db: &mut TFile, edit: LayerEdit) {
        use self::LayerEdit::*;

        // Note that we can't access the core at this point (the database implies that the core is already in use)

        match edit {
            AddKeyFrame(when) => {
                db.update(vec![
                    DatabaseUpdate::PushLayerId(self.layer_id),
                    DatabaseUpdate::PopAddKeyFrame(when)
                ])?;
            },

            RemoveKeyFrame(when) => {
                db.update(vec![
                    DatabaseUpdate::PushLayerId(self.layer_id),
                    DatabaseUpdate::PopRemoveKeyFrame(when)
                ])?;
            },

            Paint(when, edit) => {
                self.paint(db, when, edit);
            }
        }
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

    fn get_key_frames_during_time(&self, when: Range<Duration>) -> Box<Iterator<Item=Duration>> {
        let from        = when.start;
        let until       = when.end;

        let keyframes   = self.core.sync(|core| core.db.query_key_frame_times_for_layer_id(self.layer_id, from, until));

        // Turn into an iterator
        let keyframes   = keyframes.unwrap_or_else(|_: Error| vec![]);
        let keyframes   = Box::new(keyframes.into_iter());

        keyframes
    }

    fn as_vector_layer<'a>(&'a self) -> Option<Box<'a+Deref<Target='a+VectorLayer>>> {
        let vector_layer = self as &VectorLayer;

        Box::new(vector_layer)
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame> {
        let core: Result<Arc<Frame>> = self.core.sync(|core| {
            let frame               = VectorFrame::frame_at_time(&mut core.db, self.layer_id, time_index)?;
            let frame: Arc<Frame>   = Arc::new(frame);

            Ok(frame)
        });

        core.unwrap()
    }
}

impl<TFile: FloFile+Send> SqliteVectorLayer<TFile> {
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
        AnimationDbCore::insert_brush_properties(db, properties)?;

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
        AnimationDbCore::insert_brush(db, definition)?;

        // Insert the properties for this element
        db.update(vec![
            DatabaseUpdate::PopVectorBrushElement(DrawingStyleType::from(&drawing_style))
        ])?;

        Ok(())
    }

    ///
    /// Writes a brush stroke to the database (popping the element ID)
    ///
    fn create_brush_stroke(db: &mut TFile, brush_stroke: Arc<Vec<RawPoint>>) -> Result<()> {
        // TODO: we need to convert the raw points to brush points here

        db.update(vec![
            DatabaseUpdate::PopBrushPoints(brush_stroke)
        ])?;

        Ok(())
    }
}

impl<TFile: FloFile+Send+'static> VectorLayer for SqliteVectorLayer<TFile> {
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

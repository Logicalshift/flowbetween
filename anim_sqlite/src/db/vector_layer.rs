use super::*;

use animation::brushes::*;
use std::time::Duration;

///
/// Represents a vector layer in a SQLite database
/// 
pub struct SqliteVectorLayer {
    /// The ID that was assigned to this layer
    assigned_id: u64,

    /// The ID of this layer
    layer_id: i64,

    /// The type of this layer
    _layer_type: i64,

    /// The currently active brush for this layer (or none if we need to fetch this from the database)
    /// The active brush is the brush most recently added to the keyframe at the specified point in time
    active_brush: Option<(Duration, Arc<Brush>)>,

    /// Database core
    core: Arc<Desync<AnimationDbCore>>
}

///
/// Enumeration values for the vector elements
///
pub struct VectorElementEnumValues {
    pub brush_definition:   i32,
    pub brush_properties:   i32,
    pub brush_stroke:       i32
}

impl VectorElementEnumValues {
    ///
    /// Reads the enum values
    ///
    pub fn new(sqlite: &Connection) -> Result<VectorElementEnumValues> {
        // Define a function to read values
        let read_value = |name: &str| {
            sqlite.query_row(
                "SELECT Value FROM Flo_EnumerationDescriptions WHERE FieldName = \"VectorElementType\" AND ApiName = ?",
                &[&name],
                |row| row.get(0)
            )
        };

        // Read the values for the element values
        let brush_definition    = read_value("BrushDefinition")?;
        let brush_properties    = read_value("BrushProperties")?;
        let brush_stroke        = read_value("BrushStroke")?;

        // Turn into an enum values object
        Ok(VectorElementEnumValues {
            brush_definition:   brush_definition,
            brush_properties:   brush_properties,
            brush_stroke:       brush_stroke
        })
    }

    ///
    /// Retrieves the type ID for a vector element
    ///
    fn get_vector_type(&self, vector: &Vector) -> i32 {
        use animation::Vector::*;

        match vector {
            &BrushDefinition(_) => self.brush_definition,
            &BrushProperties(_) => self.brush_properties,
            &BrushStroke(_)     => self.brush_stroke
        }
    }
}

impl AnimationDb {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn get_layer_with_id(&self, assigned_id: u64) -> Option<SqliteVectorLayer> {
        SqliteVectorLayer::from_assigned_id(&self.core, assigned_id)
    }
}

impl SqliteVectorLayer {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn from_assigned_id(core: &Arc<Desync<AnimationDbCore>>, assigned_id: u64) -> Option<SqliteVectorLayer> {
        // Query for the 'real' layer ID
        let layer: Result<(i64, i64)> = core.sync(|core| {
            // Fetch the layer data (we need the 'real' ID here)
            let mut get_layer = core.sqlite.prepare(
                "SELECT Layer.LayerId, Layer.LayerType FROM Flo_AnimationLayers AS Anim \
                        INNER JOIN Flo_LayerType AS Layer ON Layer.LayerId = Anim.LayerId \
                        WHERE Anim.AnimationId = ? AND Anim.AssignedLayerId = ?;")?;
            
            let layer = get_layer.query_row(
                &[&core.animation_id, &(assigned_id as i64)],
                |layer| {
                    (layer.get(0), layer.get(1))
                }
            )?;

            Ok(layer)
        });

        // If the layer exists, create a SqliteVectorLayer
        layer.ok()
            .map(|(layer_id, layer_type)| {
                SqliteVectorLayer {
                    assigned_id:    assigned_id,
                    layer_id:       layer_id,
                    _layer_type:    layer_type,
                    active_brush:   None,
                    core:           Arc::clone(core)
                }
            })
    }
}

impl SqliteVectorLayer {
    ///
    /// Performs an async operation on the database
    /// 
    fn async<TFn: 'static+Send+Fn(&mut AnimationDbCore) -> Result<()>>(&self, action: TFn) {
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

impl Layer for SqliteVectorLayer {
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
        let keyframes = self.core.sync(|core| {
            // Query for the microsecond times from the database
            let mut get_key_frames  = core.sqlite.prepare("SELECT AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ?")?;
            let key_frames          = get_key_frames.query_map(
                &[&self.layer_id],
                |time| { let i: i64 = time.get(0); i }
            )?;

            // Convert to micros to produce the final result
            let key_frames: Vec<Duration> = key_frames
                .map(|micros| AnimationDbCore::from_micros(micros.unwrap()))
                .collect();
            
            Ok(key_frames)
        });

        // Turn into an iterator
        let keyframes = keyframes.unwrap_or_else(|_: Error| vec![]);
        let keyframes = Box::new(keyframes.into_iter());

        keyframes
    }

    fn add_key_frame(&mut self, when: Duration) {
        let layer_id = self.layer_id;

        self.async(move |core| {
            let mut insert_key_frame    = core.sqlite.prepare("INSERT INTO Flo_LayerKeyFrame (LayerId, AtTime) VALUES (?, ?)")?;
            let at_time                 = AnimationDbCore::get_micros(&when);

            insert_key_frame.execute(&[&layer_id, &at_time])?;

            Ok(())
        });
    }

    fn remove_key_frame(&mut self, when: Duration) {
        let layer_id = self.layer_id;

        self.async(move |core| {
            let mut insert_key_frame    = core.sqlite.prepare("DELETE FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime = ?")?;
            let at_time                 = AnimationDbCore::get_micros(&when);

            insert_key_frame.execute(&[&layer_id, &at_time])?;

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

impl SqliteVectorLayer {
    ///
    /// Creates a new vector element in an animation DB core
    ///
    /// The element is created without its associated data.
    ///
    fn create_new_element(core: &mut AnimationDbCore, layer_id: i64, when: Duration, element: &Vector) -> i64 {
        let mut element_id: i64 = -1;

        // Ensure that the vector enum is populated for the edit
        if core.vector_enum.is_none() {
            core.vector_enum = Some(VectorElementEnumValues::new(&core.sqlite).unwrap());
        }

        core.edit(move |sqlite, animation_id, core| {
            // Want the list of enumeration values for the vector elements
            let vector_enum = core.vector_enum.as_ref().unwrap();

            // Convert when to microseconds
            let when = AnimationDbCore::get_micros(&when);

            // SQL statements: find the frame that this time represents and insert a new element
            // We'd like to preserve these statments between calls but rusqlite imposes lifetime limits that 
            // force us to use prepare_cached (or muck around with reference objects).
            let mut get_key_frame   = sqlite.prepare_cached("SELECT KeyFrameId, AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime <= ? ORDER BY AtTime DESC LIMIT 1")?;
            let mut create_element  = sqlite.prepare_cached("INSERT INTO Flo_VectorElement (KeyFrameId, VectorElementType, AtTime) VALUES (?, ?, ?)")?;

            // Find the keyframe that we can add this element to
            let (keyframe, keyframe_time): (i64, i64) = get_key_frame.query_row(&[&layer_id, &when], |row| (row.get(0), row.get(1)))?;

            // Fetch the element type
            let element_type = vector_enum.get_vector_type(element);

            // Create the vector element
            element_id = create_element.insert(&[&keyframe, &element_type, &(when-keyframe_time)])?;

            Ok(())
        });

        // Return the element ID
        element_id
    }

    ///
    /// Writes a brush properties element to the database
    ///
    fn create_brush_properties(element_id: i64, core: &mut AnimationDbCore, properties: BrushPropertiesElement) {
        // The edit log enum needs to be loaded
        if core.edit_log_enum.is_none() {
            core.edit_log_enum = Some(EditLogEnumValues::new(&core.sqlite));
        }

        // Insert the properties for this element
        core.edit(move |sqlite, _animation_id, core| {
            // Statement to add a new brush properties entry
            let mut insert_element = sqlite.prepare_cached("INSERT INTO Flo_BrushPropertiesElement (ElementId, BrushProperties) VALUES (?, ?)")?;

            // Create the properties
            let properties_id = AnimationDbCore::insert_brush_properties(&core.sqlite, properties.brush_properties(), core.edit_log_enum.as_ref().unwrap())?;

            // Perform the insertion
            insert_element.insert(&[&element_id, &properties_id])?;

            Ok(())
        });
    }

    ///
    /// Writes a brush definition element to the database
    ///
    fn create_brush_definition(element_id: i64, core: &mut AnimationDbCore, definition: BrushDefinitionElement) {
        // The edit log enum needs to be loaded
        if core.edit_log_enum.is_none() {
            core.edit_log_enum = Some(EditLogEnumValues::new(&core.sqlite));
        }

        // Insert the properties for this element
        core.edit(move |sqlite, _animation_id, core| {
            // Statement to add a new brush properties entry
            let mut insert_element = sqlite.prepare_cached("INSERT INTO Flo_BrushElement (ElementId, Brush, DrawingStyle) VALUES (?, ?, ?)")?;

            // Create the brush
            let edit_log_enum   = core.edit_log_enum.as_ref().unwrap();
            let brush_id        = AnimationDbCore::insert_brush(&core.sqlite, definition.definition(), edit_log_enum)?;

            let drawing_style   = match definition.drawing_style() {
                BrushDrawingStyle::Draw     => edit_log_enum.draw_draw,
                BrushDrawingStyle::Erase    => edit_log_enum.draw_erase
            };

            // Perform the insertion
            insert_element.insert(&[&element_id, &brush_id, &drawing_style])?;

            Ok(())
        });
    }

    ///
    /// Writes a brush stroke to the database
    ///
    fn create_brush_stroke(element_id: i64, core: &mut AnimationDbCore, brush_stroke: BrushElement) {
        core.edit(move |sqlite, _animation_id, _core| {
            let mut insert_point = sqlite.prepare_cached("INSERT INTO Flo_BrushPoint (ElementId, PointId, X1, Y1, X2, Y2, X3, Y3, Width) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")?;

            let points  = brush_stroke.points();
            let len     = points.len();

            // Iterate through the points and add them to the database
            for (point, index) in points.iter().zip((0..len).into_iter()) {
                insert_point.insert(&[
                    &element_id,
                    &(index as i64),

                    &(point.position.0 as f64), &(point.position.1 as f64),
                    &(point.cp1.0 as f64), &(point.cp1.1 as f64),
                    &(point.cp2.0 as f64), &(point.cp2.1 as f64),
                    &(point.width as f64)
                ])?;
            }

            Ok(())
        })
    }
}

impl VectorLayer for SqliteVectorLayer {
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
            // Create a new element
            let element_id = Self::create_new_element(core, layer_id, when, &new_element);
    
            // Record the details of the element itself
            match new_element {
                BrushDefinition(brush_definition)   => Self::create_brush_definition(element_id, core, brush_definition),
                BrushProperties(brush_properties)   => Self::create_brush_properties(element_id, core, brush_properties),
                BrushStroke(brush_stroke)           => Self::create_brush_stroke(element_id, core, brush_stroke),
            }
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

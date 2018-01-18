use super::db_enum::*;
use super::db_update::*;

use rusqlite::*;
use rusqlite::types::ToSql;
use std::collections::*;
use std::time::Duration;
use std::mem;

const V1_DEFINITION: &[u8]      = include_bytes!["../../sql/flo_v1.sqlite"];
const PACKAGE_NAME: &str        = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str     = env!("CARGO_PKG_VERSION");

///
/// Provides an interface for updating and accessing the animation SQLite database
/// 
pub struct AnimationDatabase {
    /// The SQLite connection
    sqlite: Connection,

    /// The ID of the animation that we're editing
    animation_id: i64,

    /// The enum values that we know about
    enum_values: HashMap<DbEnum, i64>,

    /// The stack of IDs that we know about
    stack: Vec<i64>,

    /// None if we're not queuing updates, otherwise the list of updates that are waiting to be sent to the database
    pending: Option<Vec<DatabaseUpdate>>
}

/// List of database statements we use
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Statement {
    SelectEnumValue,
    SelectLayerId,
    SelectNearestKeyFrame,
    SelectKeyFrameTimes,
    SelectAnimationSize,
    SelectAssignedLayerIds,

    UpdateAnimationSize,

    InsertEditType,
    InsertELSetSize,
    InsertELLayer,
    InsertELWhen,
    InsertELBrush,
    InsertELBrushProperties,
    InsertELRawPoint,
    InsertBrushType,
    InsertInkBrush,
    InsertBrushProperties,
    InsertColorType,
    InsertRgb,
    InsertHsluv,
    InsertLayerType,
    InsertAssignLayer,
    InsertKeyFrame,
    InsertVectorElementType,
    InsertBrushDefinitionElement,
    InsertBrushPropertiesElement,
    InsertBrushPoint,

    DeleteKeyFrame,
    DeleteLayer
}

impl AnimationDatabase {
    ///
    /// Creates a new animation database
    /// 
    pub fn new(sqlite: Connection) -> AnimationDatabase {
        let animation_id = sqlite.query_row("SELECT MIN(AnimationId) FROM Flo_Animation", &[], |row| row.get(0)).unwrap();

        AnimationDatabase {
            sqlite:         sqlite,
            animation_id:   animation_id,
            enum_values:    HashMap::new(),
            stack:          vec![],
            pending:        None
        }
    }

    ///
    /// Initialises the database
    /// 
    pub fn setup(sqlite: &Connection) -> Result<()> {
        // Create the definition string
        let v1_definition   = String::from_utf8_lossy(V1_DEFINITION);

        // Execute against the database
        sqlite.execute_batch(&v1_definition)?;

        // Set the database version string
        let version_string      = format!("{} {}", PACKAGE_NAME, PACKAGE_VERSION);
        let mut update_version  = sqlite.prepare("UPDATE FlowBetween SET FloVersion = ?")?;
        update_version.execute(&[&version_string])?;

        Ok(())
    }

    ///
    /// Turns a microsecond count into a duration
    /// 
    fn from_micros(when: i64) -> Duration {
        Duration::new((when / 1_000_000) as u64, ((when % 1_000_000) * 1000) as u32)
    }

    ///
    /// Retrieves microseconds from a duration
    /// 
    fn get_micros(when: &Duration) -> i64 {
        let secs:i64    = when.as_secs() as i64;
        let nanos:i64   = when.subsec_nanos() as i64;

        (secs * 1_000_000) + (nanos / 1_000)
    }

    ///
    /// Returns the text of the query for a particular statements
    /// 
    fn query_for_statement(statement: Statement) -> &'static str {
        use self::Statement::*;

        match statement {
            SelectEnumValue                 => "SELECT Value FROM Flo_EnumerationDescriptions WHERE FieldName = ? AND ApiName = ?",
            SelectLayerId                   => "SELECT Layer.LayerId, Layer.LayerType FROM Flo_AnimationLayers AS Anim \
                                                       INNER JOIN Flo_LayerType AS Layer ON Layer.LayerId = Anim.LayerId \
                                                       WHERE Anim.AnimationId = ? AND Anim.AssignedLayerId = ?",
            SelectNearestKeyFrame           => "SELECT KeyFrameId, AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime <= ? ORDER BY AtTime DESC LIMIT 1",
            SelectKeyFrameTimes             => "SELECT AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ?",
            SelectAnimationSize             => "SELECT SizeX, SizeY FROM Flo_Animation WHERE AnimationId = ?",
            SelectAssignedLayerIds          => "SELECT AssignedLayerId FROM Flo_AnimationLayers WHERE AnimationId = ?",

            UpdateAnimationSize             => "UPDATE Flo_Animation SET SizeX = ?, SizeY = ? WHERE AnimationId = ?",

            InsertEditType                  => "INSERT INTO Flo_EditLog (Edit) VALUES (?)",
            InsertELSetSize                 => "INSERT INTO Flo_EL_Size (EditId, X, Y) VALUES (?, ?, ?)",
            InsertELLayer                   => "INSERT INTO Flo_EL_Layer (EditId, Layer) VALUES (?, ?)",
            InsertELWhen                    => "INSERT INTO Flo_EL_When (EditId, AtTime) VALUES (?, ?)",
            InsertELBrush                   => "INSERT INTO Flo_EL_Brush (EditId, DrawingStyle, Brush) VALUES (?, ?, ?)",
            InsertELBrushProperties         => "INSERT INTO Flo_EL_BrushProperties (EditId, BrushProperties) VALUES (?, ?)",
            InsertELRawPoint                => "INSERT INTO Flo_EL_RawPoint (EditId, PointId, PosX, PosY, Pressure, TiltX, TiltY) VALUES (?, ?, ?, ?, ?, ?, ?)",
            InsertBrushType                 => "INSERT INTO Flo_Brush_Type (BrushType) VALUES (?)",
            InsertInkBrush                  => "INSERT INTO Flo_Brush_Ink (Brush, MinWidth, MaxWidth, ScaleUpDistance) VALUES (?, ?, ?, ?)",
            InsertBrushProperties           => "INSERT INTO Flo_BrushProperties (Size, Opacity, Color) VALUES (?, ?, ?)",
            InsertColorType                 => "INSERT INTO Flo_Color_Type (ColorType) VALUES (?)",
            InsertRgb                       => "INSERT INTO Flo_Color_Rgb (Color, R, G, B) VALUES (?, ?, ?, ?)",
            InsertHsluv                     => "INSERT INTO Flo_Color_Hsluv (Color, H, S, L) VALUES (?, ?, ?, ?)",
            InsertLayerType                 => "INSERT INTO Flo_LayerType (LayerType) VALUES (?)",
            InsertAssignLayer               => "INSERT INTO Flo_AnimationLayers (AnimationId, LayerId, AssignedLayerId) VALUES (?, ?, ?)",
            InsertKeyFrame                  => "INSERT INTO Flo_LayerKeyFrame (LayerId, AtTime) VALUES (?, ?)",
            InsertVectorElementType         => "INSERT INTO Flo_VectorElement (KeyFrameId, VectorElementType, AtTime) VALUES (?, ?, ?)",
            InsertBrushDefinitionElement    => "INSERT INTO Flo_BrushElement (ElementId, Brush, DrawingStyle) VALUES (?, ?, ?)",
            InsertBrushPropertiesElement    => "INSERT INTO Flo_BrushPropertiesElement (ElementId, BrushProperties) VALUES (?, ?)",
            InsertBrushPoint                => "INSERT INTO Flo_BrushPoint (ElementId, PointId, X1, Y1, X2, Y2, X3, Y3, Width) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",

            DeleteKeyFrame                  => "DELETE FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime = ?",
            DeleteLayer                     => "DELETE FROM Flo_AnimationLayers WHERE AnimationId = ? AND LayerId = ?"
        }
    }

    ///
    /// Prepares a statement from the database
    /// 
    #[inline]
    fn prepare<'conn>(sqlite: &'conn Connection, statement: Statement) -> Result<CachedStatement<'conn>> {
        sqlite.prepare_cached(Self::query_for_statement(statement))
    }

    ///
    /// Retrieves an enum value
    /// 
    fn enum_value(&mut self, val: DbEnum) -> i64 {
        let sqlite = &self.sqlite;

        *self.enum_values.entry(val).or_insert_with(|| {
            let DbEnumName(field, name) = DbEnumName::from(val);
            Self::prepare(sqlite, Statement::SelectEnumValue)
                .unwrap()
                .query_row(&[&field, &name], |row| row.get(0))
                .unwrap()
        })
    }

    ///
    /// Executes a particular database update
    /// 
    fn execute_update(&mut self, update: DatabaseUpdate) -> Result<()> {
        use self::DatabaseUpdate::*;

        match update {
            Pop                                                             => { 
                self.stack.pop(); 
                Ok(()) 
            },

            PushEditType(edit_log_type)                                     => {
                let edit_log_type   = self.enum_value(DbEnum::EditLog(edit_log_type));
                let edit_log_id     = Self::prepare(&self.sqlite, Statement::InsertEditType)?.insert(&[&edit_log_type])?;
                self.stack.push(edit_log_id);
                Ok(())
            },

            PopEditLogSetSize(width, height)                                => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_size    = Self::prepare(&self.sqlite, Statement::InsertELSetSize)?;
                set_size.insert(&[&edit_log_id, &(width as f64), &(height as f64)])?;
                Ok(())
            },

            PushEditLogLayer(layer_id)                                      => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_layer   = Self::prepare(&self.sqlite, Statement::InsertELLayer)?;
                set_layer.insert(&[&edit_log_id, &(layer_id as i64)])?;
                self.stack.push(edit_log_id);
                Ok(())
            },

            PushEditLogWhen(when)                                           => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_when    = Self::prepare(&self.sqlite, Statement::InsertELWhen)?;
                set_when.insert(&[&edit_log_id, &Self::get_micros(&when)])?;
                self.stack.push(edit_log_id);
                Ok(())
            },

            PopEditLogBrush(drawing_style)                                  => {
                let brush_id        = self.stack.pop().unwrap();
                let edit_log_id     = self.stack.pop().unwrap();
                let drawing_style   = self.enum_value(DbEnum::DrawingStyle(drawing_style));
                let mut set_brush   = Self::prepare(&self.sqlite, Statement::InsertELBrush)?;
                set_brush.insert(&[&edit_log_id, &drawing_style, &brush_id])?;
                Ok(())
            },

            PopEditLogBrushProperties                                       => {
                let brush_props_id      = self.stack.pop().unwrap();
                let edit_log_id         = self.stack.pop().unwrap();
                let mut set_brush_props = Self::prepare(&self.sqlite, Statement::InsertELBrushProperties)?;
                set_brush_props.insert(&[&edit_log_id, &brush_props_id])?;
                Ok(())
            },

            PushRawPoints(points)                                           => {
                let edit_log_id         = self.stack.last().unwrap();
                let mut add_raw_point   = Self::prepare(&self.sqlite, Statement::InsertELRawPoint)?;
                let num_points          = points.len();

                for (point, index) in points.iter().zip((0..num_points).into_iter()) {
                    add_raw_point.insert(&[
                        edit_log_id, &(index as i64), 
                        &(point.position.0 as f64), &(point.position.1 as f64),
                        &(point.pressure as f64),
                        &(point.tilt.0 as f64), &(point.tilt.1 as f64)
                    ])?;
                }

                Ok(())                
            },

            PushBrushType(brush_type)                                       => {
                let brush_type              = self.enum_value(DbEnum::BrushDefinition(brush_type));
                let mut insert_brush_type   = Self::prepare(&self.sqlite, Statement::InsertBrushType)?;
                let brush_id                = insert_brush_type.insert(&[&brush_type])?;
                self.stack.push(brush_id);
                Ok(())
            },

            PushInkBrush(min_width, max_width, scale_up_distance)           => {
                let brush_id                = self.stack.last().unwrap();
                let mut insert_ink_brush    = Self::prepare(&self.sqlite, Statement::InsertInkBrush)?;
                insert_ink_brush.insert(&[brush_id, &(min_width as f64), &(max_width as f64), &(scale_up_distance as f64)])?;
                Ok(())
            },

            PushBrushProperties(size, opacity)                              => {
                let color_id                    = self.stack.pop().unwrap();
                let mut insert_brush_properties = Self::prepare(&self.sqlite, Statement::InsertBrushProperties)?;
                let brush_props_id              = insert_brush_properties.insert(&[&(size as f64), &(opacity as f64), &color_id])?;
                self.stack.push(brush_props_id);
                Ok(())
            },

            PushColorType(color_type)                                       => {
                let color_type              = self.enum_value(DbEnum::Color(color_type));
                let mut insert_color_type   = Self::prepare(&self.sqlite, Statement::InsertColorType)?;
                let color_id                = insert_color_type.insert(&[&color_type])?;
                self.stack.push(color_id);
                Ok(())
            },

            PushRgb(r, g, b)                                                => {
                let color_id        = self.stack.last().unwrap();
                let mut insert_rgb  = Self::prepare(&self.sqlite, Statement::InsertRgb)?;
                insert_rgb.insert(&[color_id, &(r as f64), &(g as f64), &(b as f64)])?;
                Ok(())
            },

            PushHsluv(h, s, l)                                              => {
                let color_id            = self.stack.last().unwrap();
                let mut insert_hsluv    = Self::prepare(&self.sqlite, Statement::InsertHsluv)?;
                insert_hsluv.insert(&[color_id, &(h as f64), &(s as f64), &(l as f64)])?;
                Ok(())
            },

            PopDeleteLayer                                                  => {
                let layer_id            = self.stack.pop().unwrap();
                let mut delete_layer    = Self::prepare(&self.sqlite, Statement::DeleteLayer)?;
                delete_layer.execute(&[&self.animation_id, &layer_id])?;
                Ok(())
            },

            PushLayerType(layer_type)                                       => {
                let layer_type              = self.enum_value(DbEnum::Layer(layer_type));
                let mut insert_layer_type   = Self::prepare(&self.sqlite, Statement::InsertLayerType)?;
                let layer_id                = insert_layer_type.insert(&[&layer_type])?;
                self.stack.push(layer_id);
                Ok(())
            },

            PushAssignLayer(assigned_id)                                    => {
                let layer_id                = self.stack.last().unwrap();
                let mut insert_assign_layer = Self::prepare(&self.sqlite, Statement::InsertAssignLayer)?;
                insert_assign_layer.insert(&[&self.animation_id, layer_id, &(assigned_id as i64)])?;
                Ok(())
            },

            PushLayerId(layer_id)                                           => {
                self.stack.push(layer_id);
                Ok(())
            },

            PushLayerForAssignedId(assigned_id)                             => {
                let mut select_layer_id = Self::prepare(&self.sqlite, Statement::SelectLayerId)?;
                let layer_id            = select_layer_id.query_row(&[&self.animation_id, &(assigned_id as i64)], |row| row.get(0))?;
                self.stack.push(layer_id);
                Ok(())
            },

            PopAddKeyFrame(when)                                            => {
                let layer_id                = self.stack.pop().unwrap();
                let mut insert_key_frame    = Self::prepare(&self.sqlite, Statement::InsertKeyFrame)?;
                insert_key_frame.insert(&[&layer_id, &Self::get_micros(&when)])?;
                Ok(())
            },

            PopRemoveKeyFrame(when)                                         => {
                let layer_id                = self.stack.pop().unwrap();
                let mut delete_key_frame    = Self::prepare(&self.sqlite, Statement::DeleteKeyFrame)?;
                delete_key_frame.execute(&[&layer_id, &Self::get_micros(&when)])?;
                Ok(())
            },

            PushNearestKeyFrame(when)                                       => {
                let layer_id                        = self.stack.pop().unwrap();
                let mut select_nearest_keyframe     = Self::prepare(&self.sqlite, Statement::SelectNearestKeyFrame)?;
                let (keyframe_id, start_micros)     = select_nearest_keyframe.query_row(&[&layer_id, &(Self::get_micros(&when))], |row| (row.get(0), row.get(1)))?;
                self.stack.push(start_micros);
                self.stack.push(keyframe_id);
                Ok(())
            },

            PushVectorElementType(element_type, when)                       => {
                let keyframe_id                     = self.stack.pop().unwrap();
                let start_micros                    = self.stack.pop().unwrap();
                let element_type                    = self.enum_value(DbEnum::VectorElement(element_type));
                let mut insert_vector_element_type  = Self::prepare(&self.sqlite, Statement::InsertVectorElementType)?;
                let when                            = Self::get_micros(&when) - start_micros;
                let element_id                      = insert_vector_element_type.insert(&[&keyframe_id, &element_type, &when])?;
                self.stack.push(start_micros);
                self.stack.push(keyframe_id);
                self.stack.push(element_id);
                Ok(())
            },

            PopVectorBrushElement(drawing_style)                            => {
                let brush_id                            = self.stack.pop().unwrap();
                let element_id                          = self.stack.pop().unwrap();
                let drawing_style                       = self.enum_value(DbEnum::DrawingStyle(drawing_style));
                let mut insert_brush_definition_element = Self::prepare(&self.sqlite, Statement::InsertBrushDefinitionElement)?;
                insert_brush_definition_element.insert(&[&element_id, &brush_id, &drawing_style])?;
                Ok(())
            },

            PopVectorBrushPropertiesElement                                 => {
                let brush_props_id                  = self.stack.pop().unwrap();
                let element_id                      = self.stack.pop().unwrap();
                let mut insert_brush_props_element  = Self::prepare(&self.sqlite, Statement::InsertBrushProperties)?;
                insert_brush_props_element.insert(&[&element_id, &brush_props_id])?;
                Ok(())
            },

            PopBrushPoints(points)                                          => {
                let element_id              = self.stack.pop().unwrap();
                let mut insert_brush_point  = Self::prepare(&self.sqlite, Statement::InsertBrushPoint)?;

                let num_points = points.len();
                for (point, index) in points.iter().zip((0..num_points).into_iter()) {
                    insert_brush_point.insert(&[
                        &element_id, &(index as i64),
                        &(point.cp1.0 as f64), &(point.cp1.1 as f64),
                        &(point.cp2.0 as f64), &(point.cp2.1 as f64),
                        &(point.position.0 as f64), &(point.position.1 as f64),
                        &(point.width as f64)
                    ])?;
                }

                Ok(())
            }
        }
    }

    ///
    /// Performs a set of updates on the database immediately
    /// 
    fn execute_updates_now<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<()> {
        for update in updates {
            self.execute_update(update)?;
        }
        Ok(())
    }

    ///
    /// Performs a set of updates on the database
    /// 
    pub fn update<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<()> {
        if let Some(ref mut pending) = self.pending {
            // Queue the updates into the pending queue if we're not performing them immediately
            pending.extend(updates.into_iter());
        } else {
            // Execute these updates immediately
            self.execute_updates_now(updates)?;
        }

        Ok(())
    }

    ///
    /// Starts queuing up database updates for later execution as a batch
    /// 
    pub fn begin_queuing(&mut self) {
        if self.pending.is_none() {
            self.pending = Some(vec![]);
        }
    }

    ///
    /// Executes the update queue
    /// 
    pub fn execute_queue(&mut self) -> Result<()> {
        // Fetch the pending updates
        let mut pending = None;
        mem::swap(&mut pending, &mut self.pending);

        // Execute them now
        if let Some(pending) = pending {
            self.execute_updates_now(pending)?;
        }

        Ok(())
    }

    ///
    /// Ensures any pending updates are committed to the database
    /// 
    pub fn flush_pending(&mut self) -> Result<()> {
        if self.pending.is_some() {
            // Fetch the pending updates
            let mut pending = Some(vec![]);
            mem::swap(&mut pending, &mut self.pending);

            // Execute them now
            if let Some(pending) = pending {
                self.execute_updates_now(pending)?;
            }
        }

        Ok(())
    }

    ///
    /// Queries a single row in the database
    /// 
    fn query_row<T, F: FnOnce(&Row) -> T>(&mut self, statement: Statement, params: &[&ToSql], f: F) -> Result<T> {
        self.flush_pending()?;

        let statement = Self::prepare(&self.sqlite, statement)?;
        statement.query_row(params, f)
    }

    ///
    /// Queries and maps some rows
    /// 
    fn query_map<'a, T, F: FnMut(&Row) -> T>(&'a mut self, statement: Statement, params: &[&ToSql], f: F) -> Result<MappedRows<'a, F>> {
        self.flush_pending()?;

        let statement = Self::prepare(&self.sqlite, statement)?;
        statement.query_map(params, f)
    }

    ///
    /// Finds the real layer ID for the specified assigned ID
    /// 
    pub fn query_layer_id_for_assigned_id(&mut self, assigned_id: u64) -> Result<i64> {
        self.query_row(Statement::SelectLayerId, &[&self.animation_id, &(assigned_id as i64)], |row| row.get(0))
    }

    ///
    /// Returns an iterator over the key frame times for a particular layer ID
    /// 
    pub fn query_key_frame_times_for_layer_id<'a>(&'a mut self, layer_id: i64) -> Result<Vec<Duration>> {
        let rows = self.query_map(Statement::SelectKeyFrameTimes, &[&layer_id], |row| { Self::from_micros(row.get(0)) })?;
        let rows = rows.map(|row| row.unwrap());

        Ok(rows.collect())
    }

    ///
    /// Returns the size of the animation
    /// 
    pub fn query_size(&mut self) -> Result<(f64, f64)> {
        self.query_row(Statement::SelectAnimationSize, &[&self.animation_id], |row| (row.get(0), row.get(1)))
    }

    ///
    /// Returns the assigned layer IDs
    /// 
    pub fn query_assigned_layer_ids(&mut self) -> Result<Vec<u64>> {
        let rows = self.query_map(
            Statement::SelectAssignedLayerIds, 
            &[&self.animation_id],
            |row| {
                let layer_id: i64 = row.get(0);
                layer_id as u64
            })?;

        Ok(rows.filter(|row| row.is_ok()).map(|row| row.unwrap()).collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::core::*;

    #[test]
    fn can_get_enum_value() {
        let conn = Connection::open_in_memory().unwrap();
        AnimationDatabase::setup(&conn);
        let mut db = AnimationDatabase::new(conn);

        assert!(db.enum_value(DbEnum::EditLog(EditLogType::LayerAddKeyFrame)) == 3);
    }
}
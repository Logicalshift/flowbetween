use super::db_enum::*;
use super::flo_store::*;
use super::flo_query::*;

mod query;
mod store;
pub use self::query::*;
pub use self::store::*;

use rusqlite::*;
use rusqlite::types::ToSql;
use std::collections::*;
use std::time::Duration;
use std::mem;

const V1_DEFINITION: &[u8]      = include_bytes!["../../../sql/flo_v1.sqlite"];
const PACKAGE_NAME: &str        = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str     = env!("CARGO_PKG_VERSION");

///
/// Provides an interface for updating and accessing the animation SQLite database
/// 
/// This takes a series of updates (see `DatabaseUpdate` for the list) and turns them
/// into database commands. We do things this way for a few reasons: it creates
/// an isolation layer between the implementation of the interfaces and the design
/// of the database, it provides a way to test code without actually having to
/// instantiate a database, it ensures that all of the database code is in one
/// place (making it easier to change) and it eliminates a hard dependency on
/// SQLite. (It also reduces the amount of boilerplate code needed in a lot of places)
/// 
pub struct FloSqlite {
    /// The SQLite connection
    sqlite: Connection,

    /// The ID of the animation that we're editing
    animation_id: i64,

    /// The enum values that we know about
    enum_values: HashMap<DbEnum, i64>,

    /// Maps DB enum types to maps of their values
    value_for_enum: HashMap<DbEnumType, HashMap<i64, DbEnum>>,

    /// The stack of IDs that we know about
    stack: Vec<i64>,

    /// None if we're not queuing updates, otherwise the list of updates that are waiting to be sent to the database
    pending: Option<Vec<DatabaseUpdate>>
}

/// List of database statements we use
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum FloStatement {
    SelectEnumValue,
    SelectLayerId,
    SelectNearestKeyFrame,
    SelectKeyFrameTimes,
    SelectAnimationSize,
    SelectAnimationDuration,
    SelectAnimationFrameLength,
    SelectAssignedLayerIds,
    SelectEditLogLength,
    SelectEditLogValues,
    SelectEditLogSize,
    SelectEditLogRawPoints,
    SelectColor,
    SelectBrushDefinition,
    SelectBrushProperties,
    SelectVectorElementsBefore,
    SelectBrushPoints,

    UpdateAnimationSize,

    InsertEditType,
    InsertELSetSize,
    InsertELLayer,
    InsertELWhen,
    InsertELBrush,
    InsertELBrushProperties,
    InsertELRawPoints,
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

impl FloSqlite {
    ///
    /// Creates a new animation database
    /// 
    pub fn new(sqlite: Connection) -> FloSqlite {
        let animation_id = sqlite.query_row("SELECT MIN(AnimationId) FROM Flo_Animation", &[], |row| row.get(0)).unwrap();

        FloSqlite {
            sqlite:         sqlite,
            animation_id:   animation_id,
            enum_values:    HashMap::new(),
            value_for_enum: HashMap::new(),
            stack:          vec![],
            pending:        None
        }
    }

    ///
    /// True if there are no items on the stack for this item
    /// 
    #[cfg(test)]
    pub fn stack_is_empty(&self) -> bool {
        self.stack.len() == 0
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
    /// Turns a nanosecond count into a duration
    /// 
    fn from_nanos(when: i64) -> Duration {
        Duration::new((when / 1_000_000_000) as u64, (when % 1_000_000_000) as u32)
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
    fn query_for_statement(statement: FloStatement) -> &'static str {
        use self::FloStatement::*;

        match statement {
            SelectEnumValue                 => "SELECT Value FROM Flo_EnumerationDescriptions WHERE FieldName = ? AND ApiName = ?",
            SelectLayerId                   => "SELECT Layer.LayerId, Layer.LayerType FROM Flo_AnimationLayers AS Anim \
                                                       INNER JOIN Flo_LayerType AS Layer ON Layer.LayerId = Anim.LayerId \
                                                       WHERE Anim.AnimationId = ? AND Anim.AssignedLayerId = ?",
            SelectNearestKeyFrame           => "SELECT KeyFrameId, AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime <= ? ORDER BY AtTime DESC LIMIT 1",
            SelectKeyFrameTimes             => "SELECT AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime >= ? AND AtTime < ?",
            SelectAnimationSize             => "SELECT SizeX, SizeY FROM Flo_Animation WHERE AnimationId = ?",
            SelectAnimationDuration         => "SELECT Duration FROM Flo_Animation WHERE AnimationId = ?",
            SelectAnimationFrameLength      => "SELECT Frame_Length_ns FROM Flo_Animation WHERE AnimationId = ?",
            SelectAssignedLayerIds          => "SELECT AssignedLayerId FROM Flo_AnimationLayers WHERE AnimationId = ?",
            SelectEditLogLength             => "SELECT COUNT(Id) FROM Flo_EditLog",
            SelectEditLogValues             => "SELECT EL.Id, EL.Edit, Layers.Layer, Time.AtTime, Brush.DrawingStyle, Brush.Brush, BrushProps.BrushProperties FROM Flo_EditLog AS EL \
                                                    LEFT OUTER JOIN Flo_EL_Layer           AS Layers        ON EL.Id = Layers.EditId \
                                                    LEFT OUTER JOIN Flo_EL_When            AS Time          ON EL.Id = Time.EditId \
                                                    LEFT OUTER JOIN Flo_EL_Brush           AS Brush         ON EL.Id = Brush.EditId \
                                                    LEFT OUTER JOIN Flo_EL_BrushProperties AS BrushProps    ON EL.Id = BrushProps.EditId \
                                                    LIMIT ? OFFSET ?",
            SelectEditLogSize               => "SELECT X, Y FROM Flo_EL_Size WHERE EditId = ?",
            SelectEditLogRawPoints          => "SELECT Points FROM Flo_EL_RawPoints WHERE EditId = ?",
            SelectColor                     => "SELECT Col.ColorType, Rgb.R, Rgb.G, Rgb.B, Hsluv.H, Hsluv.S, Hsluv.L FROM Flo_Color_Type AS Col \
                                                    LEFT OUTER JOIN Flo_Color_Rgb   AS Rgb      ON Col.Color = Rgb.Color \
                                                    LEFT OUTER JOIN Flo_Color_Hsluv AS Hsluv    ON Col.Color = Hsluv.Color \
                                                    WHERE Col.Color = ?",
            SelectBrushDefinition           => "SELECT Brush.BrushType, Ink.MinWidth, Ink.MaxWidth, Ink.ScaleUpDistance FROM Flo_Brush_Type AS Brush \
                                                    LEFT OUTER JOIN Flo_Brush_Ink AS Ink ON Brush.Brush = Ink.Brush \
                                                    WHERE Brush.Brush = ?",
            SelectBrushProperties           => "SELECT Size, Opacity, Color FROM Flo_BrushProperties WHERE BrushProperties = ?",
            SelectVectorElementsBefore      => "SELECT Elem.ElementId, Elem.VectorElementType, Elem.AtTime, Brush.Brush, Brush.DrawingStyle, Props.BrushProperties FROM Flo_VectorElement AS Elem \
                                                    LEFT OUTER JOIN Flo_BrushElement            AS Brush ON Elem.ElementId = Brush.ElementId \
                                                    LEFT OUTER JOIN Flo_BrushPropertiesElement  AS Props ON Elem.ElementId = Props.ElementId \
                                                    WHERE Elem.KeyFrameId = ? AND Elem.AtTime <= ? \
                                                    ORDER BY Elem.ElementId ASC",
            SelectBrushPoints               => "SELECT X1, Y1, X2, Y2, X3, Y3, Width FROM Flo_BrushPoint WHERE ElementId = ? ORDER BY PointId ASC",

            UpdateAnimationSize             => "UPDATE Flo_Animation SET SizeX = ?, SizeY = ? WHERE AnimationId = ?",

            InsertEditType                  => "INSERT INTO Flo_EditLog (Edit) VALUES (?)",
            InsertELSetSize                 => "INSERT INTO Flo_EL_Size (EditId, X, Y) VALUES (?, ?, ?)",
            InsertELLayer                   => "INSERT INTO Flo_EL_Layer (EditId, Layer) VALUES (?, ?)",
            InsertELWhen                    => "INSERT INTO Flo_EL_When (EditId, AtTime) VALUES (?, ?)",
            InsertELBrush                   => "INSERT INTO Flo_EL_Brush (EditId, DrawingStyle, Brush) VALUES (?, ?, ?)",
            InsertELBrushProperties         => "INSERT INTO Flo_EL_BrushProperties (EditId, BrushProperties) VALUES (?, ?)",
            InsertELRawPoints               => "INSERT INTO Flo_EL_RawPoints (EditId, Points) VALUES (?, ?)",
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
            DeleteLayer                     => "DELETE FROM Flo_LayerType WHERE LayerId = ?"
        }
    }

    ///
    /// Prepares a statement from the database
    /// 
    #[inline]
    fn prepare<'conn>(sqlite: &'conn Connection, statement: FloStatement) -> Result<CachedStatement<'conn>> {
        sqlite.prepare_cached(Self::query_for_statement(statement))
    }

    ///
    /// Retrieves an enum value
    /// 
    fn enum_value(&mut self, val: DbEnum) -> i64 {
        let sqlite = &self.sqlite;

        *self.enum_values.entry(val).or_insert_with(|| {
            let DbEnumName(field, name) = DbEnumName::from(val);
            Self::prepare(sqlite, FloStatement::SelectEnumValue)
                .unwrap()
                .query_row(&[&field, &name], |row| row.get(0))
                .unwrap()
        })
    }

    ///
    /// Finds the DbEnum value for a particular value
    /// 
    fn value_for_enum(&mut self, enum_type: DbEnumType, convert_value: Option<i64>) -> Option<DbEnum> {
        match convert_value {
            Some(convert_value) => {
                let sqlite = &self.sqlite;

                // Fetch/create the hash of enum values
                let enum_values = self.value_for_enum.entry(enum_type)
                    .or_insert_with(|| {
                        // Generate a hash of each value in the enum by looking them up in the database
                        let mut value_hash = HashMap::new();
                        for enum_entry in Vec::<DbEnum>::from(enum_type) {
                            // Would like to re-use self.enum_value here but can't due to borrowing rules
                            // Has the additional annoying side-effect that we can look things up twice
                            let DbEnumName(field, name) = DbEnumName::from(enum_entry);
                            let db_enum_value = Self::prepare(sqlite, FloStatement::SelectEnumValue)
                                .unwrap()
                                .query_row(&[&field, &name], |row| row.get(0))
                                .unwrap();

                            value_hash.insert(db_enum_value, enum_entry);
                        }

                        value_hash
                    });
                
                // Attempt to fetch the dbenum for the value of this type
                enum_values.get(&convert_value).map(|val| *val)
            },

            None => None
        }
    }
}

impl FloFile for FloSqlite {
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_enum_value() {
        let conn = Connection::open_in_memory().unwrap();
        FloSqlite::setup(&conn).unwrap();
        let mut db = FloSqlite::new(conn);

        assert!(db.enum_value(DbEnum::EditLog(EditLogType::LayerAddKeyFrame)) == 3);
    }
}
use super::db_enum::*;
use super::flo_store::*;
use super::flo_query::*;
use super::super::error::*;

use flo_logging::*;

use rusqlite::*;
use rusqlite::types::ToSql;
use std::collections::*;
use std::time::Duration;
use std::sync::*;
use std::mem;
use std::result::Result;

mod query;
mod store;
pub use self::query::*;
pub use self::store::*;

const V1_V2_UPGRADE: &[u8]          = include_bytes!["../../../sql/historical/flo_v1_to_v2.sqlite"];
const V3_DEFINITION: &[u8]          = include_bytes!["../../../sql/flo_v3.sqlite"];
const PACKAGE_NAME: &str            = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str         = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref V3_PATCHES: Vec<(&'static str, &'static [u8])> = vec![
        ("attached_elements", include_bytes!["../../../sql/v3_patches/attached_elements.sqlite"]),
        ("cached_drawing", include_bytes!["../../../sql/v3_patches/cached_drawing.sqlite"]),
        ("layer_cache", include_bytes!["../../../sql/v3_patches/layer_cache.sqlite"])
    ];
}

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
    /// This type's logger
    log: Arc<LogPublisher>,

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
    SelectLayerIdAndName,
    SelectElementKeyFrame,
    SelectNearestKeyFrame,
    SelectPreviousKeyFrame,
    SelectNextKeyFrame,
    SelectKeyFrameTimes,
    SelectAnimationSize,
    SelectAnimationDuration,
    SelectAnimationFrameLength,
    SelectAssignedLayerIds,
    SelectEditLogLength,
    SelectEditLogValues,
    SelectEditLogElementIds,
    SelectEditLogSize,
    SelectEditLogRawPoints,
    SelectEditLogPathId,
    SelectEditLogString,
    SelectEditLogMotionType,
    SelectColor,
    SelectBrushDefinition,
    SelectBrushProperties,
    SelectAttachmentsForElementId,
    SelectElementsForAttachmentId,
    SelectVectorElementWithId,
    SelectVectorElementTypeAssigned,
    SelectVectorElementTypeElementId,
    SelectVectorElementsBefore,
    SelectAttachedElementsBefore,
    SelectMostRecentElementOfTypeBefore,
    SelectBrushPoints,
    SelectMotion,
    SelectMotionTimePoints,
    SelectElementIdForAssignedId,
    SelectZIndexForElement,
    SelectZIndexBeforeZIndexForKeyFrame,
    SelectZIndexAfterZIndexForKeyFrame,
    SelectMaxZIndexForKeyFrame,
    SelectPathElement,
    SelectPathPointsWithTypes,
    SelectLayerCacheDrawing,

    UpdateAnimationSize,
    UpdateMotionType,
    UpdateBrushPoint,
    UpdatePathPoint,
    UpdateMoveZIndexUpwards,
    UpdateMoveZIndexDownwards,
    UpdatePathPointIndicesAfter,
    UpdatePathPointTypeIndicesAfter,

    InsertEnumValue,
    InsertEditType,
    InsertELSetSize,
    InsertELLayer,
    InsertELWhen,
    InsertELBrush,
    InsertELBrushProperties,
    InsertELElementId,
    InsertELRawPoints,
    InsertELMotionOrigin,
    InsertELMotionType,
    InsertELMotionElement,
    InsertELMotionTimePoint,
    InsertELPath,
    InsertELString,
    InsertELInt,
    InsertELFloat,
    InsertPath,
    InsertPathPoint,
    InsertPathPointType,
    InsertTimePoint,
    InsertBrushType,
    InsertInkBrush,
    InsertBrushProperties,
    InsertColorType,
    InsertRgb,
    InsertHsluv,
    InsertLayerType,
    InsertAssignLayer,
    InsertOrReplaceLayerName,
    InsertKeyFrame,
    InsertVectorElementType,
    InsertOrReplaceVectorElementTime,
    InsertOrReplaceZIndex,
    InsertElementAssignedId,
    InsertAttachElement,
    InsertBrushDefinitionElement,
    InsertBrushPropertiesElement,
    InsertBrushPoint,
    InsertPathElement,
    InsertMotion,
    InsertOrReplaceMotionOrigin,
    InsertMotionPathPoint,
    InsertNewCachedDrawing,
    InsertOrReplaceLayerCache,

    DeleteKeyFrame,
    DeleteLayer,
    DeleteElementZIndex,
    DeleteElementAttachment,
    DeleteMotion,
    DeleteMotionPoints,
    DeleteLayerCache,
    DeletePathPointRange,
    DeletePathPointTypeRange,
    DeleteVectorElement,
    DeleteVectorElementTime
}

impl FloSqlite {
    ///
    /// Creates a new animation database. The connection must already have been initialized via `setup`.
    ///
    pub fn new(sqlite: Connection) -> FloSqlite {
        let mut sqlite = sqlite;
        Self::upgrade(&mut sqlite).unwrap();

        let animation_id = sqlite.query_row("SELECT MIN(AnimationId) FROM Flo_Animation", NO_PARAMS, |row| row.get(0)).unwrap();

        FloSqlite {
            log:            Arc::new(LogPublisher::new(module_path!())),
            sqlite:         sqlite,
            animation_id:   animation_id,
            enum_values:    HashMap::new(),
            value_for_enum: HashMap::new(),
            stack:          vec![],
            pending:        None
        }
    }

    ///
    /// Upgrades a connection so that it conforms to the latest version
    ///
    fn upgrade(sqlite: &mut Connection) -> Result<(), SqliteAnimationError> {
        let animation_version: i64 = sqlite.query_row("SELECT DataVersion FROM FlowBetween", NO_PARAMS, |row| row.get(0))?;

        if animation_version == 1 {
            Self::upgrade_v1_to_v2(sqlite)?;
        } else if animation_version == 2 {
            return Err(SqliteAnimationError::CannotUpgradeVersionTooOld(animation_version));
        } else if animation_version == 3 {
            Self::apply_v3_patches(sqlite)?;
        } else {
            return Err(SqliteAnimationError::UnsupportedVersionNumber(animation_version));
        }

        Ok(())
    }

    ///
    /// Upgrades a version 1 to a version 2 database
    ///
    fn upgrade_v1_to_v2(sqlite: &mut Connection) -> Result<(), SqliteAnimationError> {
        let v2_upgrade  = String::from_utf8_lossy(V1_V2_UPGRADE);
        sqlite.execute_batch(&v2_upgrade)?;

        Ok(())
    }

    ///
    /// Applies patches to ensure that a v3 file format database is up to date
    ///
    fn apply_v3_patches(sqlite: &mut Connection) -> Result<(), SqliteAnimationError> {
        let patch_transaction = sqlite.transaction()?;

        // Apply the patches that we know about
        for (patch_name, patch_sql) in V3_PATCHES.iter() {
            // See if this patch has already been applied
            let num_patches = patch_transaction.query_row("SELECT COUNT(*) FROM Flo_AppliedPatches WHERE PatchName = ?;", &[*patch_name], |row| row.get::<_, i64>(0))?;

            if num_patches == 0 {
                // Apply the patch if it does not already exist
                let patch_sql       = String::from_utf8_lossy(patch_sql);
                patch_transaction.execute_batch(&patch_sql)?;

                // Add to the 'applied patches' table so this patch is not re-applied
                let version_string  = format!("{} {}", PACKAGE_NAME, PACKAGE_VERSION);
                patch_transaction.execute::<&[&dyn ToSql]>("INSERT INTO Flo_AppliedPatches (PatchName, PatchSql, AppliedByVersion) VALUES (?, ?, ?);", &[patch_name, &patch_sql, &version_string])?;
            }
        }

        // Check for patches that we can't understand (indicate that this database might be from a newer version of FlowBetween)
        {
            let mut all_patches     = patch_transaction.prepare("SELECT PatchName FROM Flo_AppliedPatches")?;
            let all_patches         = all_patches
                .query_map(NO_PARAMS, |row| row.get::<_, String>(0))?
                .map(|row| row.unwrap_or_else(|err| format!("<< error: {:?} >>", err)))
                .collect::<HashSet<_>>();

            for (patch_name, _patch_sql) in V3_PATCHES.iter() {
                if !all_patches.contains(*patch_name) {
                    // All patches must be supported by this version of the tool
                    return Err(SqliteAnimationError::UnsupportedFormatPatch(String::from(*patch_name)));
                }
            }
        }

        // Finished patching
        patch_transaction.commit()?;

        // Updated OK
        Ok(())
    }

    ///
    /// True if there are no items on the stack for this item
    ///
    #[cfg(test)]
    pub fn stack_is_empty(&self) -> bool {
        self.stack.len() == 0
    }

    ///
    /// Initialises the database as new
    ///
    pub fn setup(sqlite: &Connection) -> Result<(), SqliteAnimationError> {
        // Create the definition string
        let definition      = String::from_utf8_lossy(V3_DEFINITION);

        // Execute against the database
        sqlite.execute_batch(&definition)?;

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
            SelectEnumValue                     => "SELECT Value FROM Flo_EnumerationDescriptions WHERE FieldName = ? AND ApiName = ?",
            SelectLayerId                       => "SELECT Layer.LayerId, Layer.LayerType FROM Flo_AnimationLayers AS Anim \
                                                        INNER JOIN Flo_LayerType AS Layer ON Layer.LayerId = Anim.LayerId \
                                                        WHERE Anim.AnimationId = ? AND Anim.AssignedLayerId = ?",
            SelectLayerIdAndName                => "SELECT Layer.LayerId, Layer.LayerType, LayerName.Name FROM Flo_AnimationLayers AS Anim \
                                                        INNER JOIN Flo_LayerType AS Layer ON Layer.LayerId = Anim.LayerId \
                                                        LEFT OUTER JOIN Flo_LayerName AS LayerName ON Layer.LayerId = LayerName.LayerId \
                                                        WHERE Anim.AnimationId = ? AND Anim.AssignedLayerId = ?",
            SelectNearestKeyFrame               => "SELECT KeyFrameId, AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime <= ? ORDER BY AtTime DESC LIMIT 1",
            SelectElementKeyFrame               => "SELECT KeyFrameId FROM Flo_VectorElementTime WHERE ElementId = ?",
            SelectPreviousKeyFrame              => "SELECT KeyFrameId, AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime < ? ORDER BY AtTime DESC LIMIT 1",
            SelectNextKeyFrame                  => "SELECT KeyFrameId, AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime > ? ORDER BY AtTime ASC LIMIT 1",
            SelectKeyFrameTimes                 => "SELECT AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime >= ? AND AtTime < ?",
            SelectAnimationSize                 => "SELECT SizeX, SizeY FROM Flo_Animation WHERE AnimationId = ?",
            SelectAnimationDuration             => "SELECT Duration FROM Flo_Animation WHERE AnimationId = ?",
            SelectAnimationFrameLength          => "SELECT Frame_Length_ns FROM Flo_Animation WHERE AnimationId = ?",
            SelectAssignedLayerIds              => "SELECT AssignedLayerId FROM Flo_AnimationLayers WHERE AnimationId = ?",
            SelectEditLogLength                 => "SELECT COUNT(Id) FROM Flo_EditLog",
            SelectEditLogValues                 => "SELECT EL.Id, EL.Edit, Layers.Layer, Time.AtTime, Brush.DrawingStyle, Brush.Brush, BrushProps.BrushProperties, ElementId.ElementId FROM Flo_EditLog AS EL \
                                                        LEFT OUTER JOIN Flo_EL_Layer           AS Layers        ON EL.Id = Layers.EditId \
                                                        LEFT OUTER JOIN Flo_EL_When            AS Time          ON EL.Id = Time.EditId \
                                                        LEFT OUTER JOIN Flo_EL_Brush           AS Brush         ON EL.Id = Brush.EditId \
                                                        LEFT OUTER JOIN Flo_EL_BrushProperties AS BrushProps    ON EL.Id = BrushProps.EditId \
                                                        LEFT OUTER JOIN Flo_EL_ElementIds      AS ElementId     ON EL.Id = ElementId.EditId \
                                                        WHERE ElementId.ElementIndex = 0 OR ElementId.ElementIndex IS NULL \
                                                        LIMIT ? OFFSET ?",
            SelectEditLogElementIds             => "SELECT ElementId.ElementId FROM Flo_EL_ElementIds AS ElementId WHERE ElementId.EditId = ? ORDER BY ElementId.ElementIndex",
            SelectEditLogSize                   => "SELECT X, Y FROM Flo_EL_Size WHERE EditId = ?",
            SelectEditLogRawPoints              => "SELECT Points FROM Flo_EL_RawPoints WHERE EditId = ?",
            SelectEditLogPathId                 => "SELECT PathId FROM Flo_EL_Path WHERE EditId = ?",
            SelectEditLogString                 => "SELECT String FROM Flo_EL_StringParameters WHERE EditId = ? AND StringIndex = ?",
            SelectEditLogMotionType             => "SELECT MotionType FROM Flo_EL_MotionType WHERE EditId = ?",

            SelectColor                         => "SELECT Col.ColorType, Rgb.R, Rgb.G, Rgb.B, Hsluv.H, Hsluv.S, Hsluv.L FROM Flo_Color_Type AS Col \
                                                        LEFT OUTER JOIN Flo_Color_Rgb   AS Rgb      ON Col.Color = Rgb.Color \
                                                        LEFT OUTER JOIN Flo_Color_Hsluv AS Hsluv    ON Col.Color = Hsluv.Color \
                                                        WHERE Col.Color = ?",
            SelectBrushDefinition               => "SELECT Brush.BrushType, Ink.MinWidth, Ink.MaxWidth, Ink.ScaleUpDistance FROM Flo_Brush_Type AS Brush \
                                                        LEFT OUTER JOIN Flo_Brush_Ink AS Ink ON Brush.Brush = Ink.Brush \
                                                        WHERE Brush.Brush = ?",
            SelectAttachmentsForElementId       => "SELECT Attch.AttachedElementId, Elem.VectorElementType, Assgn.AssignedId FROM Flo_ElementAttachments AS Attch \
                                                        INNER JOIN Flo_VectorElement            AS Elem     ON Elem.ElementId = Attch.AttachedElementId \
                                                        LEFT OUTER JOIN Flo_AssignedElementId   AS Assgn    ON Elem.ElementId = Assgn.ElementId \
                                                        WHERE Attch.ElementId = ?;",
            SelectElementsForAttachmentId       => "SELECT Attch.ElementId, Elem.VectorElementType, Assgn.AssignedId FROM Flo_ElementAttachments AS Attch \
                                                        INNER JOIN Flo_VectorElement            AS Elem     ON Elem.ElementId = Attch.ElementId \
                                                        LEFT OUTER JOIN Flo_AssignedElementId   AS Assgn    ON Elem.ElementId = Assgn.ElementId \
                                                        WHERE Attch.AttachedElementId = ?;",
            SelectBrushProperties               => "SELECT Size, Opacity, Color FROM Flo_BrushProperties WHERE BrushProperties = ?",
            SelectVectorElementWithId           => "SELECT Elem.ElementId, Elem.VectorElementType, Time.AtTime, Brush.Brush, Brush.DrawingStyle, Props.BrushProperties, Assgn.AssignedId
                                                        FROM Flo_VectorElement                      AS Elem \
                                                        LEFT OUTER JOIN Flo_VectorElementTime       AS Time  ON Elem.ElementId = Time.ElementId \
                                                        LEFT OUTER JOIN Flo_BrushElement            AS Brush ON Elem.ElementId = Brush.ElementId \
                                                        LEFT OUTER JOIN Flo_BrushPropertiesElement  AS Props ON Elem.ElementId = Props.ElementId \
                                                        LEFT OUTER JOIN Flo_AssignedElementId       AS Assgn ON Elem.ElementId = Assgn.ElementId \
                                                        WHERE Elem.ElementId = ?",
            SelectVectorElementTypeAssigned     => "SELECT Elem.VectorElementType FROM Flo_VectorElement    AS Elem \
                                                        INNER JOIN Flo_AssignedElementId                    AS Assgn    ON Assgn.ElementId = Elem.ElementId \
                                                        WHERE Assgn.AssignedId = ?",
            SelectVectorElementTypeElementId    => "SELECT Elem.VectorElementType FROM Flo_VectorElement    AS Elem \
                                                        WHERE Elem.ElementId = ?",
            SelectVectorElementsBefore          => "SELECT Elem.ElementId, Elem.VectorElementType, Time.AtTime, Brush.Brush, Brush.DrawingStyle, Props.BrushProperties, Assgn.AssignedId
                                                        FROM Flo_VectorElement                      AS Elem \
                                                        INNER JOIN Flo_VectorElementTime            AS Time  ON Elem.ElementId = Time.ElementId \
                                                        LEFT OUTER JOIN Flo_BrushElement            AS Brush ON Elem.ElementId = Brush.ElementId \
                                                        LEFT OUTER JOIN Flo_BrushPropertiesElement  AS Props ON Elem.ElementId = Props.ElementId \
                                                        LEFT OUTER JOIN Flo_AssignedElementId       AS Assgn ON Elem.ElementId = Assgn.ElementId \
                                                        LEFT OUTER JOIN Flo_VectorElementOrdering   AS Ordr  ON Elem.ElementId = Ordr.ElementId AND Time.KeyFrameId = Ordr.KeyFrameId \
                                                        WHERE Time.KeyFrameId = ? AND Time.AtTime <= ? \
                                                        ORDER BY Ordr.ZIndex ASC, Elem.ElementId ASC",
            SelectAttachedElementsBefore        => "WITH RECURSIVE \
                                                        AttachedElement AS ( \
                                                            SELECT Elem.ElementId AS ElementId, NULL AS ParentElementId, Elem.VectorElementType AS ElementType \
                                                                FROM Flo_VectorElement              AS Elem \
                                                                INNER JOIN Flo_VectorElementTime    AS Time  ON Elem.ElementId = Time.ElementId \
                                                                WHERE Time.KeyFrameId = ? AND Time.AtTime <= ? \
                                                            UNION
                                                            SELECT Elem.ElementId AS ElementId, AttachedElement.ElementId AS ParentElementId, Elem.VectorElementType AS ElementType \
                                                                FROM AttachedElement
                                                                INNER JOIN Flo_ElementAttachments   AS Attch ON Attch.ElementId = AttachedElement.ElementId \
                                                                INNER JOIN Flo_VectorElement        AS Elem ON Elem.ElementId = Attch.AttachedElementId
                                                        ) \
                                                    SELECT Elem.ParentElementId, Elem.ElementId, Elem.ElementType, Time.AtTime, Brush.Brush, Brush.DrawingStyle, Props.BrushProperties, Assgn.AssignedId, Ordr.ZIndex, ParentAssgn.AssignedId \
                                                        FROM AttachedElement                        AS Elem
                                                        LEFT OUTER JOIN Flo_VectorElementTime       AS Time         ON Elem.ElementId = Time.ElementId \
                                                        LEFT OUTER JOIN Flo_BrushElement            AS Brush        ON Elem.ElementId = Brush.ElementId \
                                                        LEFT OUTER JOIN Flo_BrushPropertiesElement  AS Props        ON Elem.ElementId = Props.ElementId \
                                                        LEFT OUTER JOIN Flo_AssignedElementId       AS Assgn        ON Elem.ElementId = Assgn.ElementId \
                                                        LEFT OUTER JOIN Flo_AssignedElementId       AS ParentAssgn  ON Elem.ParentElementId = ParentAssgn.ElementId \
                                                        LEFT OUTER JOIN Flo_VectorElementOrdering   AS Ordr         ON Elem.ElementId = Ordr.ElementId AND Time.KeyFrameId = Ordr.KeyFrameId \
                                                        ORDER BY Ordr.ZIndex ASC",
            SelectMostRecentElementOfTypeBefore => "SELECT Elem.ElementId, Elem.VectorElementType, Time.AtTime, Brush.Brush, Brush.DrawingStyle, Props.BrushProperties, Assgn.AssignedId \
                                                        FROM Flo_VectorElement                      AS Elem \
                                                        INNER JOIN Flo_VectorElementTime            AS Time  ON Elem.ElementId = Time.ElementId \
                                                        LEFT OUTER JOIN Flo_BrushElement            AS Brush ON Elem.ElementId = Brush.ElementId \
                                                        LEFT OUTER JOIN Flo_BrushPropertiesElement  AS Props ON Elem.ElementId = Props.ElementId \
                                                        LEFT OUTER JOIN Flo_AssignedElementId       AS Assgn ON Elem.ElementId = Assgn.ElementId \
                                                        WHERE Time.KeyFrameId = ? AND Time.AtTime <= ? AND Elem.VectorElementType = ? \
                                                        ORDER BY Time.AtTime DESC \
                                                        LIMIT 1",
            SelectBrushPoints                   => "SELECT X1, Y1, X2, Y2, X3, Y3, Width FROM Flo_BrushPoint WHERE ElementId = ? ORDER BY PointId ASC",
            SelectMotion                        => "SELECT Mot.MotionType, Origin.X, Origin.Y \
                                                        FROM Flo_Motion                     AS Mot
                                                        LEFT OUTER JOIN Flo_MotionOrigin    AS Origin ON Mot.MotionId = Origin.MotionId
                                                        WHERE Mot.MotionId = ?",
            SelectMotionTimePoints              => "SELECT Point.X, Point.Y, Point.Milliseconds \
                                                        FROM Flo_MotionPath         AS Path \
                                                        INNER JOIN Flo_TimePoint    AS Point ON Path.PointId = Point.PointId \
                                                        WHERE Path.MotionId = ? AND Path.PathType = ? \
                                                        ORDER BY Path.PointIndex ASC",
            SelectElementIdForAssignedId        => "SELECT ElementId FROM Flo_AssignedElementId WHERE AssignedId = ?",
            SelectZIndexForElement              => "SELECT ZIndex FROM Flo_VectorElementOrdering WHERE ElementId = ?",
            SelectZIndexBeforeZIndexForKeyFrame => "SELECT IFNULL(MAX(ZIndex), 0) FROM Flo_VectorElementOrdering WHERE KeyFrameId = ? AND ZIndex < ?",
            SelectZIndexAfterZIndexForKeyFrame  => "SELECT IFNULL(MIN(ZIndex), 0) FROM Flo_VectorElementOrdering WHERE KeyFrameId = ? AND ZIndex > ?",
            SelectMaxZIndexForKeyFrame          => "SELECT IFNULL(MAX(ZIndex), 0) FROM Flo_VectorElementOrdering WHERE KeyFrameId = ?",
            SelectPathElement                   => "SELECT Elem.PathId \
                                                        FROM Flo_PathElement    AS Elem \
                                                        WHERE Elem.ElementId = ?",
            SelectPathPointsWithTypes           => "SELECT Path.X, Path.Y, Types.Type FROM Flo_PathPointType AS Types \
                                                        LEFT OUTER JOIN Flo_PathPoints AS Path ON (Path.PathId = Types.PathId AND Types.PointIndex = Path.PointIndex) \
                                                        WHERE Types.PathId = ? \
                                                        ORDER BY Types.PointIndex ASC",
            SelectLayerCacheDrawing             => "SELECT Draw.Drawing FROM Flo_LayerCache AS Cache \
                                                        INNER JOIN Flo_CachedDrawings AS Draw ON Cache.CacheId = Draw.CacheId \
                                                        WHERE Cache.CacheType = ? AND Cache.LayerId = ? AND Cache.CacheTime = ?;",

            UpdateAnimationSize                 => "UPDATE Flo_Animation SET SizeX = ?, SizeY = ? WHERE AnimationId = ?",
            UpdateMotionType                    => "UPDATE Flo_Motion SET MotionType = ? WHERE MotionId = ?",
            UpdateBrushPoint                    => "UPDATE Flo_BrushPoint SET X1 = ?, Y1 = ?, X2 = ?, Y2 = ?, X3 = ?, Y3 = ? WHERE ElementId = ? AND PointId = ?",
            UpdatePathPoint                     => "UPDATE Flo_PathPoints SET X = ?, Y = ? WHERE PathId = ? AND PointIndex = ?",
            UpdateMoveZIndexUpwards             => "UPDATE Flo_VectorElementOrdering SET ZIndex = ZIndex + 1 WHERE KeyFrameId = ? AND ZIndex >= ?",
            UpdateMoveZIndexDownwards           => "UPDATE Flo_VectorElementOrdering SET ZIndex = ZIndex - 1 WHERE KeyFrameId = ? AND ZIndex >= ?",
            UpdatePathPointIndicesAfter         => "UPDATE Flo_PathPoints SET PointIndex = PointIndex + ? WHERE PathId = ? AND PointIndex >= ?",
            UpdatePathPointTypeIndicesAfter     => "UPDATE Flo_PathPointType SET PointIndex = PointIndex + ? WHERE PathId = ? AND PointIndex >= ?",

            InsertEnumValue                     => "INSERT INTO Flo_EnumerationDescriptions (FieldName, Value, ApiName, Comment) SELECT ?, (SELECT IFNULL(Max(Value)+1, 0) FROM Flo_EnumerationDescriptions WHERE FieldName = ?), ?, ?",
            InsertEditType                      => "INSERT INTO Flo_EditLog (Edit) VALUES (?)",
            InsertELSetSize                     => "INSERT INTO Flo_EL_Size (EditId, X, Y) VALUES (?, ?, ?)",
            InsertELLayer                       => "INSERT INTO Flo_EL_Layer (EditId, Layer) VALUES (?, ?)",
            InsertELWhen                        => "INSERT INTO Flo_EL_When (EditId, AtTime) VALUES (?, ?)",
            InsertELBrush                       => "INSERT INTO Flo_EL_Brush (EditId, DrawingStyle, Brush) VALUES (?, ?, ?)",
            InsertELBrushProperties             => "INSERT INTO Flo_EL_BrushProperties (EditId, BrushProperties) VALUES (?, ?)",
            InsertELElementId                   => "INSERT INTO Flo_EL_ElementIds (EditId, ElementIndex, ElementId) VALUES (?, ?, ?)",
            InsertELRawPoints                   => "INSERT INTO Flo_EL_RawPoints (EditId, Points) VALUES (?, ?)",
            InsertELMotionOrigin                => "INSERT INTO Flo_EL_MotionOrigin (EditId, X, Y) VALUES (?, ?, ?)",
            InsertELMotionType                  => "INSERT INTO Flo_EL_MotionType (EditId, MotionType) VALUES (?, ?)",
            InsertELMotionElement               => "INSERT INTO Flo_EL_MotionAttach (EditId, AttachedElement) VALUES (?, ?)",
            InsertELMotionTimePoint             => "INSERT INTO Flo_EL_MotionPath (EditId, PointIndex, TimePointId) VALUES (?, ?, ?)",
            InsertELPath                        => "INSERT INTO Flo_EL_Path (EditId, PathId) VALUES (?, ?)",
            InsertELString                      => "INSERT INTO Flo_EL_StringParameters (EditId, StringIndex, String) VALUES (?, ?, ?)",
            InsertELInt                         => "INSERT INTO Flo_EL_IntParameters (EditId, IntIndex, Value) VALUES (?, ?, ?)",
            InsertELFloat                       => "INSERT INTO Flo_EL_FloatParameters (EditId, FloatIndex, Value) VALUES (?, ?, ?)",
            InsertPath                          => "INSERT INTO Flo_Path (PathId) VALUES (NULL)",
            InsertPathPoint                     => "INSERT INTO Flo_PathPoints (PathId, PointIndex, X, Y) VALUES (?, ?, ?, ?)",
            InsertPathPointType                 => "INSERT INTO Flo_PathPointType (PathId, PointIndex, Type) VALUES (?, ?, ?)",
            InsertTimePoint                     => "INSERT INTO Flo_TimePoint (X, Y, Milliseconds) VALUES (?, ?, ?)",
            InsertBrushType                     => "INSERT INTO Flo_Brush_Type (BrushType) VALUES (?)",
            InsertInkBrush                      => "INSERT INTO Flo_Brush_Ink (Brush, MinWidth, MaxWidth, ScaleUpDistance) VALUES (?, ?, ?, ?)",
            InsertBrushProperties               => "INSERT INTO Flo_BrushProperties (Size, Opacity, Color) VALUES (?, ?, ?)",
            InsertColorType                     => "INSERT INTO Flo_Color_Type (ColorType) VALUES (?)",
            InsertRgb                           => "INSERT INTO Flo_Color_Rgb (Color, R, G, B) VALUES (?, ?, ?, ?)",
            InsertHsluv                         => "INSERT INTO Flo_Color_Hsluv (Color, H, S, L) VALUES (?, ?, ?, ?)",
            InsertLayerType                     => "INSERT INTO Flo_LayerType (LayerType) VALUES (?)",
            InsertAssignLayer                   => "INSERT INTO Flo_AnimationLayers (AnimationId, LayerId, AssignedLayerId) VALUES (?, ?, ?)",
            InsertOrReplaceLayerName            => "INSERT OR REPLACE INTO Flo_LayerName (LayerId, Name) VALUES (?, ?)",
            InsertKeyFrame                      => "INSERT INTO Flo_LayerKeyFrame (LayerId, AtTime) VALUES (?, ?)",
            InsertVectorElementType             => "INSERT INTO Flo_VectorElement (VectorElementType) VALUES (?)",
            InsertOrReplaceVectorElementTime    => "INSERT OR REPLACE INTO Flo_VectorElementTime (ElementId, KeyFrameId, AtTime) VALUES (?, ?, ?)",
            InsertOrReplaceZIndex               => "INSERT OR REPLACE INTO Flo_VectorElementOrdering (ElementId, KeyFrameId, ZIndex) VALUES (?, ?, ?)",
            InsertBrushDefinitionElement        => "INSERT INTO Flo_BrushElement (ElementId, Brush, DrawingStyle) VALUES (?, ?, ?)",
            InsertBrushPropertiesElement        => "INSERT INTO Flo_BrushPropertiesElement (ElementId, BrushProperties) VALUES (?, ?)",
            InsertBrushPoint                    => "INSERT INTO Flo_BrushPoint (ElementId, PointId, X1, Y1, X2, Y2, X3, Y3, Width) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            InsertElementAssignedId             => "INSERT INTO Flo_AssignedElementId (ElementId, AssignedId) VALUES (?, ?)",
            InsertAttachElement                 => "INSERT OR IGNORE INTO Flo_ElementAttachments (ElementId, AttachedElementId) VALUES (?, ?)",
            InsertPathElement                   => "INSERT INTO Flo_PathElement (ElementId, PathId) VALUES (?, ?)",
            InsertMotion                        => "INSERT INTO Flo_Motion (MotionId, MotionType) VALUES (?, ?)",
            InsertOrReplaceMotionOrigin         => "INSERT OR REPLACE INTO Flo_MotionOrigin (MotionId, X, Y) VALUES (?, ?, ?)",
            InsertMotionPathPoint               => "INSERT INTO Flo_MotionPath (MotionId, PathType, PointIndex, PointId) VALUES (?, ?, ?, ?)",
            InsertNewCachedDrawing              => "INSERT INTO Flo_CachedDrawings (Drawing) VALUES (?)",
            InsertOrReplaceLayerCache           => "INSERT OR REPLACE INTO Flo_LayerCache (CacheType, LayerId, CacheTime, CacheId) VALUES (?, ?, ?, ?)",

            DeleteKeyFrame                      => "DELETE FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime = ?",
            DeleteLayer                         => "DELETE FROM Flo_LayerType WHERE LayerId = ?",
            DeleteElementZIndex                 => "DELETE FROM Flo_VectorElementOrdering WHERE ElementId = ?",
            DeleteElementAttachment             => "DELETE FROM Flo_ElementAttachments WHERE ElementId = ? AND AttachedElementId = ?",
            DeleteMotion                        => "DELETE FROM Flo_Motion WHERE MotionId = ?",
            DeleteMotionPoints                  => "DELETE FROM Flo_MotionPath WHERE MotionId = ? AND PathType = ?",
            DeleteLayerCache                    => "DELETE FROM Flo_LayerCache WHERE CacheType = ? AND LayerId = ? AND CacheTime = ?",
            DeletePathPointRange                => "DELETE FROM Flo_PathPoints WHERE PathId = ? AND PointIndex >= ? AND PointIndex < ?",
            DeletePathPointTypeRange            => "DELETE FROM Flo_PathPointType WHERE PathId = ? AND PointIndex >= ? AND PointIndex < ?",
            DeleteVectorElement                 => "DELETE FROM Flo_VectorElement WHERE ElementId = ?",
            DeleteVectorElementTime             => "DELETE FROM Flo_VectorElementTime WHERE ElementId = ?"
        }
    }

    ///
    /// Prepares a statement from the database
    ///
    #[inline]
    fn prepare<'conn>(sqlite: &'conn Connection, statement: FloStatement) -> Result<CachedStatement<'conn>, SqliteAnimationError> {
        Ok(sqlite.prepare_cached(Self::query_for_statement(statement))?)
    }

    ///
    /// Retrieves an enum value
    ///
    fn enum_value(&mut self, val: DbEnum) -> i64 {
        let sqlite = &self.sqlite;

        *self.enum_values.entry(val).or_insert_with(|| {
            let DbEnumName(field, name) = DbEnumName::from(val);

            // Try to retrieve this value from the database
            let existing_value = Self::prepare(sqlite, FloStatement::SelectEnumValue)
                .unwrap()
                .query_row(&[&field, &name], |row| row.get(0));

            if let Err(Error::QueryReturnedNoRows) = existing_value {
                // If the value doesn't exist, try to insert it as a new value
                Self::prepare(sqlite, FloStatement::InsertEnumValue)
                    .unwrap()
                    .insert::<&[&dyn ToSql]>(&[&field, &field, &name, &String::from("")])
                    .unwrap();

                // Try again to fetch the row
                Self::prepare(sqlite, FloStatement::SelectEnumValue)
                    .unwrap()
                    .query_row(&[&field, &name], |row| row.get(0))
                    .unwrap()
            } else {
                // Result is the existing value
                existing_value.unwrap()
            }
        })
    }

    ///
    /// Finds the DbEnum value for a particular value
    ///
    fn value_for_enum(&mut self, enum_type: DbEnumType, convert_value: Option<i64>) -> Option<DbEnum> {
        match convert_value {
            Some(convert_value)     => {
                // Fetch/create the hash of enum values
                let enum_values = if self.value_for_enum.contains_key(&enum_type) {
                    // Use cached version
                    self.value_for_enum.get(&enum_type).unwrap()
                } else {
                    // Generate a hash of each value in the enum by looking them up in the database
                    let mut value_hash = HashMap::new();
                    for enum_entry in Vec::<DbEnum>::from(enum_type) {
                        let db_enum_value = self.enum_value(enum_entry);

                        value_hash.insert(db_enum_value, enum_entry);
                    }

                    self.value_for_enum.insert(enum_type, value_hash);

                    // Final result is the value we just cached
                    self.value_for_enum.get(&enum_type).unwrap()
                };

                // Attempt to fetch the dbenum for the value of this type
                enum_values.get(&convert_value).map(|val| *val)
            },

            None    => None
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

        // Enum values are created starting at 0
        assert!(db.enum_value(DbEnum::EditLog(EditLogType::LayerAddKeyFrame)) == 0);
        assert!(db.enum_value(DbEnum::EditLog(EditLogType::LayerRemoveKeyFrame)) == 1);

        // They're independent for different enum types
        assert!(db.enum_value(DbEnum::DrawingStyle(DrawingStyleType::Draw)) == 0);
    }
}

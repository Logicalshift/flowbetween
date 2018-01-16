use super::db_enum::*;
use super::db_update::*;

use rusqlite::*;
use std::collections::*;

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
    /// Returns the text of the query for a particular statements
    /// 
    fn query_for_statement(statement: Statement) -> &'static str {
        use self::Statement::*;

        match statement {
            SelectEnumValue                 => "SELECT Value FROM Flo_EnumerationDescriptions WHERE FieldName = ? AND ApiName = ?",
            SelectLayerId                   => "SELECT Layer.LayerId, Layer.LayerType FROM Flo_AnimationLayers AS Anim \
                                                       INNER JOIN Flo_LayerType AS Layer ON Layer.LayerId = Anim.LayerId \
                                                       WHERE Anim.AnimationId = ? AND Anim.AssignedLayerId = ?",

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
            DeleteLayer                     => "DELETE FROM Flo_AnimationLayers WHERE AssignedLayerId = ?"
        }
    }

    ///
    /// Prepares a statement from the database
    /// 
    #[inline]
    fn prepare<'conn>(&'conn self, statement: Statement) -> Result<CachedStatement<'conn>> {
        self.sqlite.prepare_cached(Self::query_for_statement(statement))
    }

    ///
    /// Fetches a statement from a cache, or prepares it
    /// 
    fn prepare_with_cache<'a, 'conn>(&'conn self, statement: Statement, cache: &'a mut HashMap<Statement, CachedStatement<'conn>>) -> &'a mut CachedStatement<'conn> {
        cache.entry(statement).or_insert_with(|| self.prepare(statement).unwrap())
    }

    ///
    /// Executes a particular database update
    /// 
    fn execute_update(<'a, 'conn>(&'conn self, update: DatabaseUpdate, cache: &'a mut HashMap<Statement, CachedStatement<'conn>>) {
        unimplemented!()
    }
}
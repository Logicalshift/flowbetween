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

    /// The enum values that we know about
    enum_values: HashMap<DbEnum, i64>,

    /// The stack of IDs that we know about
    stack: Vec<i64>
}

/// List of database statements we use
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
enum Statement {
    SelectEnumValue,

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

    DeleteKeyFrame
}

impl AnimationDatabase {
    ///
    /// Creates a new animation database
    /// 
    pub fn new(sqlite: Connection) -> AnimationDatabase {
        AnimationDatabase {
            sqlite:         sqlite,
            enum_values:    HashMap::new(),
            stack:          vec![]
        }
    }

    ///
    /// Returns the text of the query for a particular statements
    /// 
    fn query_for_statement(statement: Statement) -> &'static str {
        match statement {
            _ => unimplemented!()
        }
    }
}
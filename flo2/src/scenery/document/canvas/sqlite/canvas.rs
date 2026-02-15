use super::id_cache::*;
use super::super::error::*;
use super::super::layer::*;
use super::super::property::*;
use super::super::shape::*;
use super::super::shape_type::*;

use rusqlite::*;

use std::collections::{HashMap};
use std::result::{Result};

/// Definition for the canvas sqlite storage
pub (super) static SCHEMA: &'static str = include_str!("canvas.sql");

///
/// Storage for the sqlite canvas
///
pub struct SqliteCanvas {
    /// Connection to the sqlite database
    pub (super) sqlite: Connection,

    /// Cache of the known property IDs
    pub (super) property_id_cache: HashMap<CanvasPropertyId, i64>,

    /// Reverse cache of the known property IDs
    pub (super) property_for_id_cache: HashMap<i64, CanvasPropertyId>,

    /// Cache of the known shape type IDs
    pub (super) shapetype_id_cache: HashMap<ShapeType, i64>,

    /// Reverse cache of the known shape type IDs
    pub (super) shapetype_for_id_cache: HashMap<i64, ShapeType>,

    /// Cache of the known shape IDs (maps to the index for the shape)
    pub (super) shape_id_cache: IdCache<CanvasShapeId, i64>,

    /// Cache of the known layer IDs (maps to the index for the layer)
    pub (super) layer_id_cache: HashMap<CanvasLayerId, i64>,

    /// The next shape ID to use (None if we haven't retrieved this from the database yet)
    pub (super) next_shape_id: Option<i64>,
}

impl SqliteCanvas {
    ///
    /// Creates a storage structure with an existing connection
    ///
    pub fn with_connection(sqlite: Connection) -> Result<Self, CanvasError> {
        sqlite.execute_batch("PRAGMA foreign_keys = ON")?;

        Ok(Self {
            sqlite:                 sqlite,
            property_id_cache:      HashMap::new(),
            property_for_id_cache:  HashMap::new(),
            shapetype_id_cache:     HashMap::new(),
            shapetype_for_id_cache: HashMap::new(),
            shape_id_cache:         IdCache::new(200),
            layer_id_cache:         HashMap::new(),
            next_shape_id:          None,
        })
    }

    ///
    /// Initialises the canvas in this object
    ///
    pub fn initialise(&self) -> Result<(), CanvasError> {
        self.sqlite.execute_batch(SCHEMA)?;

        Ok(())
    }

    ///
    /// Creates a new SQLite canvas in memory
    ///
    pub fn new_in_memory() -> Result<Self, CanvasError> {
        let sqlite  = Connection::open_in_memory()?;
        let canvas  = Self::with_connection(sqlite)?;
        canvas.initialise()?;

        Ok(canvas)
    }
}

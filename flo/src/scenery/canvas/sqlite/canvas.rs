// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::id_cache::*;
use super::super::document_properties::*;
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
    pub fn initialise(&mut self) -> Result<(), CanvasError> {
        self.sqlite.execute_batch(SCHEMA)?;

        let default_props: Vec<&dyn ToCanvasProperties> = vec![&DocumentSize { width: 1920.0, height: 1080.0 }, &DocumentTimePerFrame(1.0/12.0)];
        self.set_properties(CanvasPropertyTarget::Document, default_props.to_properties()).expect("Initial properties should be set OK");

        Ok(())
    }

    ///
    /// Creates a new SQLite canvas in memory
    ///
    pub fn new_in_memory() -> Result<Self, CanvasError> {
        let sqlite      = Connection::open_in_memory()?;
        let mut canvas  = Self::with_connection(sqlite)?;
        canvas.initialise()?;

        Ok(canvas)
    }
}

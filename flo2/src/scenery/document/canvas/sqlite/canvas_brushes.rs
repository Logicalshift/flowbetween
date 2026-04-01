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

use super::canvas::*;
use super::super::brush::*;
use super::super::error::*;
use super::super::property::*;

use rusqlite::*;

use std::result::{Result};

impl SqliteCanvas {
    ///
    /// Queries the database for the ordering index of the specified layer
    ///
    #[inline]
    pub fn index_for_brush(&mut self, brush_id: CanvasBrushId) -> Result<i64, CanvasError> {
        Ok(self.sqlite.query_one::<i64, _, _>("SELECT BrushId FROM Brushes WHERE BrushGuid = ?", [brush_id.to_string()], |row| row.get(0))?)
    }

    ///
    /// Updates the properties for a brush
    ///
    pub fn set_brush_properties(&mut self, brush_id: CanvasBrushId, properties: Vec<(CanvasPropertyId, CanvasProperty)>) -> Result<(), CanvasError> {
        let brush_idx = self.index_for_brush(brush_id)?;

        // Map to property IDs
        let properties = properties.into_iter()
            .map(|(property_id, property)| self.index_for_property(property_id).map(move |int_id| (int_id, property)))
            .collect::<Result<Vec<_>, _>>()?;

        // Write the properties themselves
        let transaction = self.sqlite.transaction()?;

        // Run commands to set each type of property value
        let mut blob_properties_cmd = transaction.prepare_cached("REPLACE INTO BrushBlobProperties (BrushId, PropertyId, BlobValue) VALUES (?, ?, ?)")?;
        Self::set_blob_properties(&properties, &mut blob_properties_cmd, vec![&brush_idx])?;

        drop(blob_properties_cmd);

        transaction.commit()?;

        Ok(())
    }

    ///
    /// Adds a brush to the canvas
    ///
    pub fn add_brush(&mut self, brush_id: CanvasBrushId) -> Result<(), CanvasError> {
        self.sqlite.execute("INSERT INTO Brushes (BrushGuid) VALUES (?)", params![brush_id.to_string()])?;

        Ok(())
    }

    ///
    /// Removes a brush and all its associations from the canvas
    ///
    pub fn remove_brush(&mut self, brush_id: CanvasBrushId) -> Result<(), CanvasError> {
        self.sqlite.execute("DELETE FROM Brushes WHERE BrushGuid = ?", params![brush_id.to_string()])?;

        Ok(())
    }
}

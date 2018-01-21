use super::*;

use animation::*;

impl FloSqlite {
    ///
    /// Queries a single row in the database
    /// 
    fn query_row<T, F: FnOnce(&Row) -> T>(&mut self, statement: FloStatement, params: &[&ToSql], f: F) -> Result<T> {
        self.flush_pending()?;

        let mut statement = Self::prepare(&self.sqlite, statement)?;
        statement.query_row(params, f)
    }

    ///
    /// Queries and maps some rows
    /// 
    fn query_map<'a, T: 'a, F: FnMut(&Row) -> T>(&mut self, statement: FloStatement, params: &[&ToSql], f: F) -> Result<Box<'a+Iterator<Item=Result<T>>>> {
        self.flush_pending()?;

        // Prepare the statement
        let mut statement = Self::prepare(&self.sqlite, statement)?;

        // Gather the results into a vector (can't keep the map due to lifetime requirements: Rust can't preserve the statement outside of this function)
        let results: Vec<Result<T>> = statement.query_map(params, f)?.collect();

        // Convert into an iterator (into_iter preserves the lifetime of the vec so we don't have the same problem)
        Ok(Box::new(results.into_iter()))
    }
}

impl FloQuery for FloSqlite {
    ///
    /// Finds the real layer ID for the specified assigned ID
    /// 
    fn query_layer_id_for_assigned_id(&mut self, assigned_id: u64) -> Result<i64> {
        let animation_id = self.animation_id;
        self.query_row(FloStatement::SelectLayerId, &[&animation_id, &(assigned_id as i64)], |row| row.get(0))
    }

    ///
    /// Returns an iterator over the key frame times for a particular layer ID
    /// 
    fn query_key_frame_times_for_layer_id<'a>(&'a mut self, layer_id: i64) -> Result<Vec<Duration>> {
        let rows = self.query_map(FloStatement::SelectKeyFrameTimes, &[&layer_id], |row| { Self::from_micros(row.get(0)) })?;
        let rows = rows.map(|row| row.unwrap());

        Ok(rows.collect())
    }


    ///
    /// Queries the nearest keyframe to the specified time in the specified layer
    /// 
    fn query_nearest_key_frame<'a>(&'a mut self, layer_id: i64, when: Duration) -> Result<(i64, Duration)> {
        self.query_row(FloStatement::SelectNearestKeyFrame, &[&layer_id, &Self::get_micros(&when)], |row| (row.get(0), Self::from_micros(row.get(1))))
    }

    ///
    /// Returns the size of the animation
    /// 
    fn query_size(&mut self) -> Result<(f64, f64)> {
        let animation_id = self.animation_id;
        self.query_row(FloStatement::SelectAnimationSize, &[&animation_id], |row| (row.get(0), row.get(1)))
    }

    ///
    /// Returns the assigned layer IDs
    /// 
    fn query_assigned_layer_ids(&mut self) -> Result<Vec<u64>> {
        let animation_id = self.animation_id;
        let rows = self.query_map(
            FloStatement::SelectAssignedLayerIds, 
            &[&animation_id],
            |row| {
                let layer_id: i64 = row.get(0);
                layer_id as u64
            })?;

        Ok(rows.filter(|row| row.is_ok()).map(|row| row.unwrap()).collect())
    }

    ///
    /// Retrieves the total number of entries in the edit log
    /// 
    fn query_edit_log_length(&mut self) -> Result<i64> {
        self.query_row(FloStatement::SelectEditLogLength, &[], |row| row.get(0))
    }

    ///
    /// Retrieves a set of values from the edit log
    /// 
    fn query_edit_log_values(&mut self, from_index: i64, to_index: i64) -> Result<Vec<EditLogEntry>> {
        // Converts an i64 from the DB to an u64 as we use those for IDs
        #[inline]
        fn as_id(id_in: Option<i64>) -> Option<u64> {
            match id_in {
                Some(id_in) => Some(id_in as u64),
                None        => None
            }
        }

        // Converts an i64 to a duration
        #[inline]
        fn as_duration(time_in: Option<i64>) -> Option<Duration> {
            match time_in {
                Some(time_in)   => Some(FloSqlite::from_micros(time_in)),
                None            => None
            }
        }

        // Fetch the entries from the database
        // Can't call value_for_enum from query_map due to lifetimes, and need to deal
        // with the fact that individual rows can have errors as well as the whole thing,
        // so this ends up messy
        self.query_map(FloStatement::SelectEditLogValues, &[&(to_index-from_index), &(from_index)],
            |row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5), row.get(6)))
            .map(|rows_with_errors| rows_with_errors
                .map(|row_with_error| row_with_error.unwrap())
                .map(|(edit_id, edit_type, layer_id, when, drawing_style, brush_id, brush_properties_id)| {
                    let edit_type       = self.value_for_enum(DbEnumType::EditLog, Some(edit_type)).unwrap().edit_log().unwrap();
                    let drawing_style   = self.value_for_enum(DbEnumType::DrawingStyle, drawing_style).and_then(|ds| ds.drawing_style());

                    let brush_id: Option<i64>               = brush_id;
                    let brush_properties_id: Option<i64>    = brush_properties_id;

                    EditLogEntry {
                        edit_id:                edit_id,
                        edit_type:              edit_type,
                        layer_id:               as_id(layer_id),
                        when:                   as_duration(when),
                        brush:                  brush_id.and_then(|brush_id| drawing_style.and_then(|drawing_style| Some((brush_id, drawing_style)))),
                        brush_properties_id:    brush_properties_id
                    }
                }).collect()
            )
    }

    ///
    /// Queries the size associated with an edit log entry
    /// 
    fn query_edit_log_size(&mut self, edit_id: i64) -> Result<(f64, f64)> {
        self.query_row(FloStatement::SelectEditLogSize, &[&edit_id], |row| {
            (row.get(0), row.get(1))
        })
    }

    ///
    /// Retrieves the raw points associated with a particular edit ID
    /// 
    fn query_edit_log_raw_points(&mut self, edit_id: i64) -> Result<Vec<RawPoint>> {
        self.query_map(FloStatement::SelectEditLogRawPoints, &[&edit_id], |row| {
            let position: (f64, f64)    = (row.get(0), row.get(1));
            let pressure: f64           = row.get(2);
            let tilt: (f64, f64)        = (row.get(3), row.get(4));

            RawPoint {
                position:   (position.0 as f32, position.1 as f32),
                pressure:   pressure as f32,
                tilt:       (tilt.0 as f32, tilt.1 as f32)
            }
        })
        .map(|rows_with_errors| rows_with_errors.map(|row| row.unwrap()).collect())
    }

    ///
    /// Retrieves a colour with the specified ID
    /// 
    fn query_color(&mut self, color_id: i64) -> Result<ColorEntry> {
        self.query_row(FloStatement::SelectColor, &[&color_id], |row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5), row.get(6)))
            .map(|(color_type, r, g, b, h, s, l)| {
                let r: Option<f64>  = r;
                let g: Option<f64>  = g;
                let b: Option<f64>  = b;
                let h: Option<f64>  = h;
                let s: Option<f64>  = s;
                let l: Option<f64>  = l;
                let color_type      = self.value_for_enum(DbEnumType::Color, Some(color_type)).and_then(|color_type| color_type.color());

                ColorEntry {
                    color_type: color_type.unwrap(),

                    rgb:        r.and_then(|r| g.map(|g| (r, g))).and_then(|(r, g)| b.map(|b| (r, g, b))),
                    hsluv:      h.and_then(|h| s.map(|s| (h, s))).and_then(|(h, s)| l.map(|l| (h, s, l)))
                }
            })
    }

    ///
    /// Retrieves the brush with the specified ID
    /// 
    fn query_brush(&mut self, brush_id: i64) -> Result<BrushEntry> {
        self.query_row(FloStatement::SelectBrushDefinition, &[&brush_id], |row| (row.get(0), row.get(1), row.get(2), row.get(3)))
            .map(|(brush_type, min_width, max_width, scale_up_distance)| {
                let min_width: Option<f64>          = min_width;
                let max_width: Option<f64>          = max_width;
                let scale_up_distance: Option<f64>  = scale_up_distance;
                let brush_type                      = self.value_for_enum(DbEnumType::BrushDefinition, Some(brush_type)).and_then(|brush_type| brush_type.brush_definition());

                BrushEntry {
                    brush_type: brush_type.unwrap(),
                    ink_defn:   min_width.and_then(|min_width| max_width.map(|max_width| (min_width, max_width))).and_then(|(min_width, max_width)| scale_up_distance.map(|scale_up| (min_width, max_width, scale_up)))
                }
            })
    }

    ///
    /// Retrieves the brush properties with the specified ID
    /// 
    fn query_brush_properties(&mut self, brush_properties_id: i64) -> Result<BrushPropertiesEntry> {
        self.query_row(FloStatement::SelectBrushProperties, &[&brush_properties_id], |row| {
            BrushPropertiesEntry {
                size:       row.get(0),
                opacity:    row.get(1),
                color_id:   row.get(2)
            }
        })
    }

    ///
    /// Queries the vector elements that appear before a certain time in the specified keyframe
    /// 
    fn query_vector_keyframe_elements_before(&mut self, keyframe_id: i64, before: Duration) -> Result<Vec<VectorElementEntry>> {
        // Can't call value_for_enum from query_map due to lifetimes, and need to deal
        // with the fact that individual rows can have errors as well as the whole thing,
        // so this ends up messy
        self.query_map(FloStatement::SelectVectorElementsBefore, &[&keyframe_id, &Self::get_micros(&before)], |row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5)))
            .map(|rows_with_errors|
                rows_with_errors.map(|row_with_error| row_with_error.unwrap())
                    .map(|(element_id, element_type, when, brush_id, drawing_style, brush_properties_id)| {
                        let when                    = Self::from_micros(when);
                        let brush_id: Option<i64>   = brush_id;
                        let drawing_style           = self.value_for_enum(DbEnumType::DrawingStyle, drawing_style).and_then(|drawing_style| drawing_style.drawing_style());
                        let element_type            = self.value_for_enum(DbEnumType::VectorElement, Some(element_type)).unwrap().vector_element().unwrap();

                        let brush                   = brush_id.and_then(|brush_id| drawing_style.map(|drawing_style| (brush_id, drawing_style)));

                        VectorElementEntry {
                            element_id,
                            element_type,
                            when,
                            brush,
                            brush_properties_id
                        }
                    })
                    .collect())
    }


    ///
    /// Queries the brush points associated with a vector element
    /// 
    fn query_vector_element_brush_points(&mut self, element_id: i64) -> Result<Vec<BrushPoint>> {
        self.query_map(FloStatement::SelectBrushPoints, &[&element_id],
            |row| {
                let x1:     f64 = row.get(0);
                let y1:     f64 = row.get(1);
                let x2:     f64 = row.get(2);
                let y2:     f64 = row.get(3);
                let x3:     f64 = row.get(4);
                let y3:     f64 = row.get(5);
                let width:  f64 = row.get(6);

                BrushPoint {
                    cp1:        (x1 as f32, y1 as f32),
                    cp2:        (x2 as f32, y2 as f32),
                    position:   (x3 as f32, y3 as f32),
                    width:      width as f32
                }
            })
            .map(|rows_with_errors| rows_with_errors.map(|row_with_error| row_with_error.unwrap()).collect())
    }
}

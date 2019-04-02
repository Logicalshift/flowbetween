use super::*;
use super::super::motion_path_type::*;

use flo_animation::*;

impl FloSqlite {
    ///
    /// Queries a single row in the database
    /// 
    fn query_row<T, F: FnOnce(&Row) -> T>(&mut self, statement: FloStatement, params: &[&dyn ToSql], f: F) -> Result<T, SqliteAnimationError> {
        self.flush_pending()?;

        let mut statement = Self::prepare(&self.sqlite, statement)?;
        Ok(statement.query_row(params, f)?)
    }

    ///
    /// Queries and maps some rows
    /// 
    fn query_map<'a, T: 'a, F: FnMut(&Row) -> T>(&mut self, statement: FloStatement, params: &[&dyn ToSql], f: F) -> Result<Box<dyn 'a+Iterator<Item=Result<T, SqliteAnimationError>>>, SqliteAnimationError> {
        self.flush_pending()?;

        // Prepare the statement
        let mut statement = Self::prepare(&self.sqlite, statement)?;

        // Gather the results into a vector (can't keep the map due to lifetime requirements: Rust can't preserve the statement outside of this function)
        let results: Vec<_> = statement.query_map(params, f)?.collect();

        // Convert into an iterator (into_iter preserves the lifetime of the vec so we don't have the same problem)
        Ok(Box::new(results.into_iter().map(|err| err.map_err(|err| err.into()))))
    }
}

impl FloQuery for FloSqlite {
    ///
    /// Finds the real layer ID for the specified assigned ID
    /// 
    fn query_layer_id_for_assigned_id(&mut self, assigned_id: u64) -> Result<(i64, Option<String>), SqliteAnimationError> {
        let animation_id = self.animation_id;
        self.query_row(FloStatement::SelectLayerIdAndName, &[&animation_id, &(assigned_id as i64)], |row| (row.get(0), row.get(2)))
    }

    ///
    /// Returns an iterator over the key frame times for a particular layer ID
    /// 
    fn query_key_frame_times_for_layer_id(&mut self, layer_id: i64, from: Duration, until: Duration) -> Result<Vec<Duration>, SqliteAnimationError> {
        let from    = Self::get_micros(&from);
        let until   = Self::get_micros(&until);

        let rows    = self.query_map(FloStatement::SelectKeyFrameTimes, &[&layer_id, &from, &until], |row| { Self::from_micros(row.get(0)) })?;
        let rows    = rows.map(|row| row.unwrap());

        Ok(rows.collect())
    }

    ///
    /// Queries the nearest keyframe to the specified time in the specified layer
    /// 
    fn query_nearest_key_frame<'a>(&'a mut self, layer_id: i64, when: Duration) -> Result<Option<(i64, Duration)>, SqliteAnimationError> {
        let res = self.query_row(FloStatement::SelectNearestKeyFrame, &[&layer_id, &Self::get_micros(&when)], |row| (row.get(0), Self::from_micros(row.get(1))));

        match res {
            Err(SqliteAnimationError::QueryReturnedNoRows)  => Ok(None),
            other                                           => Ok(Some(other?))
        }
    }

    ///
    /// Similar to query_nearest_key_frame except finds the previous and next keyframes instead
    /// 
    fn query_previous_and_next_key_frame(&mut self, layer_id: i64, when: Duration) -> Result<(Option<(i64, Duration)>, Option<(i64, Duration)>), SqliteAnimationError> {
        let when            = Self::get_micros(&when);

        // Allow a 1ms buffer for the 'current' frame
        let previous_when   = when - 1000;
        let next_when       = when + 1000;

        // Query for the previous and next keyframe
        let previous        = self.query_row(FloStatement::SelectPreviousKeyFrame, &[&layer_id, &previous_when], |row| (row.get(0), Self::from_micros(row.get(1))));
        let next            = self.query_row(FloStatement::SelectNextKeyFrame, &[&layer_id, &next_when], |row| (row.get(0), Self::from_micros(row.get(1))));

        // The 'no rows' query result just means 'no match'
        let previous        = match previous {
            Err(SqliteAnimationError::QueryReturnedNoRows)  => None,
            other                                           => Some(other?)
        };

        let next            = match next {
            Err(SqliteAnimationError::QueryReturnedNoRows)  => None,
            other                                           => Some(other?)
        };

        // Return the previous and next frames that we found
        Ok((previous, next))
    }

    ///
    /// Returns the size of the animation
    /// 
    fn query_size(&mut self) -> Result<(f64, f64), SqliteAnimationError> {
        let animation_id = self.animation_id;
        self.query_row(FloStatement::SelectAnimationSize, &[&animation_id], |row| (row.get(0), row.get(1)))
    }

    ///
    /// Returns the total length of the animation
    /// 
    fn query_duration(&mut self) -> Result<Duration, SqliteAnimationError> {
        let animation_id = self.animation_id;
        self.query_row(FloStatement::SelectAnimationDuration, &[&animation_id], |row| Self::from_micros(row.get(0)))
    }

    ///
    /// Returns the length of a frame in the animation
    /// 
    fn query_frame_length(&mut self) -> Result<Duration, SqliteAnimationError> {
        let animation_id = self.animation_id;
        self.query_row(FloStatement::SelectAnimationFrameLength, &[&animation_id], |row| Self::from_nanos(row.get(0)))
    }

    ///
    /// Returns the assigned layer IDs
    /// 
    fn query_assigned_layer_ids(&mut self) -> Result<Vec<u64>, SqliteAnimationError> {
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
    fn query_edit_log_length(&mut self) -> Result<i64, SqliteAnimationError> {
        self.query_row(FloStatement::SelectEditLogLength, &[], |row| row.get(0))
    }

    ///
    /// Retrieves a set of values from the edit log
    /// 
    fn query_edit_log_values(&mut self, from_index: i64, to_index: i64) -> Result<Vec<EditLogEntry>, SqliteAnimationError> {
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
            |row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5), row.get(6), row.get(7)))
            .map(|rows_with_errors| rows_with_errors
                .map(|row_with_error| row_with_error.unwrap())
                .map(|(edit_id, edit_type, layer_id, when, drawing_style, brush_id, brush_properties_id, element_id)| {
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
                        brush_properties_id:    brush_properties_id,
                        element_id:             element_id
                    }
                }).collect()
            )
    }

    ///
    /// Queries the size associated with an edit log entry
    /// 
    fn query_edit_log_size(&mut self, edit_id: i64) -> Result<(f64, f64), SqliteAnimationError> {
        self.query_row(FloStatement::SelectEditLogSize, &[&edit_id], |row| {
            (row.get(0), row.get(1))
        })
    }

    ///
    /// Retrieves the raw points associated with a particular edit ID
    /// 
    fn query_edit_log_raw_points(&mut self, edit_id: i64) -> Result<Vec<RawPoint>, SqliteAnimationError> {
        self.query_row(FloStatement::SelectEditLogRawPoints, &[&edit_id], |row| {
            let point_bytes: Vec<_>     = row.get(0);
            let mut point_bytes: &[u8]  = &point_bytes;

            read_raw_points(&mut point_bytes).unwrap()
        })
    }
    
    ///
    /// Retrieves the ID of the path associated with the specified edit ID
    ///
    fn query_edit_log_path_id(&mut self, edit_id: i64) -> Result<i64, SqliteAnimationError> {
        self.query_row(FloStatement::SelectEditLogPathId, &[&edit_id], |row| {
            row.get(0)
        })
    }

    ///
    /// Retrieves the string associated with a specific edit ID
    ///
    fn query_edit_log_string(&mut self, edit_id: i64, string_index: u32) -> Result<String, SqliteAnimationError> {
        let string_index = string_index as i64;
        self.query_row(FloStatement::SelectEditLogString, &[&edit_id, &string_index], |row| {
            row.get(0)
        })
    }

    ///
    /// Retrieves a colour with the specified ID
    /// 
    fn query_color(&mut self, color_id: i64) -> Result<ColorEntry, SqliteAnimationError> {
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
    fn query_brush(&mut self, brush_id: i64) -> Result<BrushEntry, SqliteAnimationError> {
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
    fn query_brush_properties(&mut self, brush_properties_id: i64) -> Result<BrushPropertiesEntry, SqliteAnimationError> {
        self.query_row(FloStatement::SelectBrushProperties, &[&brush_properties_id], |row| {
            BrushPropertiesEntry {
                size:       row.get(0),
                opacity:    row.get(1),
                color_id:   row.get(2)
            }
        })
    }

    ///
    /// Retrieves the vector element with the specified ID
    ///
    fn query_vector_element(&mut self, id: i64) -> Result<VectorElementEntry, SqliteAnimationError> {
        self.query_row(FloStatement::SelectVectorElementWithId, &[&id], |row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5), row.get(6)))
            .map(|(element_id, element_type, when, brush_id, drawing_style, brush_properties_id, assigned_id)| {
                let assigned_id: Option<i64> = assigned_id;
                let when: Option<i64>       = when;
                let when                    = when.map(|when| Self::from_micros(when));
                let brush_id: Option<i64>   = brush_id;
                let drawing_style           = self.value_for_enum(DbEnumType::DrawingStyle, drawing_style).and_then(|drawing_style| drawing_style.drawing_style());
                let element_type            = self.value_for_enum(DbEnumType::VectorElement, Some(element_type)).unwrap().vector_element().unwrap();
                let assigned_id             = ElementId::from(assigned_id);

                let brush                   = brush_id.and_then(|brush_id| drawing_style.map(|drawing_style| (brush_id, drawing_style)));

                VectorElementEntry {
                    element_id,
                    element_type,
                    when,
                    brush,
                    brush_properties_id,
                    assigned_id
                }
            })
    }

    ///
    /// Queries the vector elements that appear before a certain time in the specified keyframe
    /// 
    fn query_vector_keyframe_elements_before(&mut self, keyframe_id: i64, before: Duration) -> Result<Vec<VectorElementEntry>, SqliteAnimationError> {
        self.query_map(FloStatement::SelectVectorElementsBefore, &[&keyframe_id, &Self::get_micros(&before)], |row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5), row.get(6)))
            .map(|rows_with_errors|
                rows_with_errors.map(|row_with_error| row_with_error.unwrap())
                    .map(|(element_id, element_type, when, brush_id, drawing_style, brush_properties_id, assigned_id)| {
                        let assigned_id: Option<i64> = assigned_id;
                        let when                    = Some(Self::from_micros(when));
                        let brush_id: Option<i64>   = brush_id;
                        let drawing_style           = self.value_for_enum(DbEnumType::DrawingStyle, drawing_style).and_then(|drawing_style| drawing_style.drawing_style());
                        let element_type            = self.value_for_enum(DbEnumType::VectorElement, Some(element_type)).unwrap().vector_element().unwrap();
                        let assigned_id             = ElementId::from(assigned_id);

                        let brush                   = brush_id.and_then(|brush_id| drawing_style.map(|drawing_style| (brush_id, drawing_style)));

                        VectorElementEntry {
                            element_id,
                            element_type,
                            when,
                            brush,
                            brush_properties_id,
                            assigned_id
                        }
                    })
                    .collect())
    }

    ///
    /// Queries the vector elements and all attachments that appear before a certain time in the specified keyframe
    ///
    fn query_vector_keyframe_elements_and_attachments_before(&mut self, keyframe_id: i64, before: Duration) -> Result<Vec<VectorElementAttachmentEntry>, SqliteAnimationError> {
        self.query_map(FloStatement::SelectAttachedElementsBefore, &[&keyframe_id, &Self::get_micros(&before)], |row| (row.get(0), row.get(1), row.get(2), row.get::<_, Option<i64>>(3), row.get(4), row.get(5), row.get(6), row.get(7), row.get::<_, Option<i64>>(8), row.get(9)))
            .map(|rows_with_errors|
                rows_with_errors.map(|row_with_error| row_with_error.unwrap())
                    .map(|(parent_element_id, element_id, element_type, when, brush_id, drawing_style, brush_properties_id, assigned_id, z_index, parent_assigned_id)| {
                        let parent_element_id: Option<i64>  = parent_element_id;
                        let parent_assigned_id: Option<i64> = parent_assigned_id;
                        let assigned_id: Option<i64>        = assigned_id;
                        let when                            = when.map(|when| Self::from_micros(when));
                        let brush_id: Option<i64>           = brush_id;
                        let drawing_style                   = self.value_for_enum(DbEnumType::DrawingStyle, drawing_style).and_then(|drawing_style| drawing_style.drawing_style());
                        let element_type                    = self.value_for_enum(DbEnumType::VectorElement, Some(element_type)).unwrap().vector_element().unwrap();
                        let assigned_id                     = ElementId::from(assigned_id);
                        let parent_assigned_id              = ElementId::from(parent_assigned_id);

                        let brush                           = brush_id.and_then(|brush_id| drawing_style.map(|drawing_style| (brush_id, drawing_style)));

                        let vector_element = VectorElementEntry {
                            element_id,
                            element_type,
                            when,
                            brush,
                            brush_properties_id,
                            assigned_id
                        };

                        VectorElementAttachmentEntry {
                            attached_to_element:        parent_element_id,
                            attached_to_assigned_id:    parent_assigned_id,
                            vector:                     vector_element,
                            z_index:                    z_index
                        }
                    })
                    .collect())
    }

    ///
    /// Queries the single most recent element of the specified type in the specified keyframe
    /// 
    fn query_most_recent_element_of_type(&mut self, keyframe_id: i64, before: Duration, element_type: VectorElementType) -> Result<Option<VectorElementEntry>, SqliteAnimationError> {
        let element_type = self.enum_value(DbEnum::VectorElement(element_type));

        // Can't call value_for_enum from query_map due to lifetimes, and need to deal
        // with the fact that individual rows can have errors as well as the whole thing,
        // so this ends up messy
        self.query_map(FloStatement::SelectMostRecentElementOfTypeBefore, &[&keyframe_id, &Self::get_micros(&before), &element_type], |row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4), row.get(5), row.get(6)))
            .map(|rows_with_errors|
                rows_with_errors.map(|row_with_error| row_with_error.unwrap())
                    .map(|(element_id, element_type, when, brush_id, drawing_style, brush_properties_id, assigned_id)| {
                        let assigned_id: Option<i64> = assigned_id;
                        let when                    = Some(Self::from_micros(when));
                        let brush_id: Option<i64>   = brush_id;
                        let drawing_style           = self.value_for_enum(DbEnumType::DrawingStyle, drawing_style).and_then(|drawing_style| drawing_style.drawing_style());
                        let element_type            = self.value_for_enum(DbEnumType::VectorElement, Some(element_type)).unwrap().vector_element().unwrap();
                        let assigned_id             = ElementId::from(assigned_id);

                        let brush                   = brush_id.and_then(|brush_id| drawing_style.map(|drawing_style| (brush_id, drawing_style)));

                        VectorElementEntry {
                            element_id,
                            element_type,
                            when,
                            brush,
                            brush_properties_id,
                            assigned_id
                        }
                    })
                    .nth(0))
    }


    ///
    /// Retrieves the element ID from an assigned ID
    ///
    fn query_vector_element_id(&mut self, assigned_id: &ElementId) -> Result<Option<i64>, SqliteAnimationError> {
        if let Some(assigned_id) = assigned_id.id() {
            Ok(Some(self.query_row(FloStatement::SelectElementIdForAssignedId, &[&assigned_id], |row| row.get(0))?))
        } else {
            Ok(None)
        }
    }

    ///
    /// Queries the type of a single vector element given its assigned ID
    /// 
    fn query_vector_element_type_from_assigned_id(&mut self, assigned_id: i64) -> Result<Option<VectorElementType>, SqliteAnimationError> {
        self.query_row(FloStatement::SelectVectorElementTypeAssigned, &[&assigned_id], |row| row.get(0))
            .map(|element_type| {
                let element_type    = self.value_for_enum(DbEnumType::VectorElement, Some(element_type)).unwrap().vector_element().unwrap();

                Some(element_type)
            })
    }

    ///
    /// Queries the type of a single vector element given its element id
    /// 
    fn query_vector_element_type_from_element_id(&mut self, element_id: i64) -> Result<Option<VectorElementType>, SqliteAnimationError> {
        self.query_row(FloStatement::SelectVectorElementTypeElementId, &[&element_id], |row| row.get(0))
            .map(|element_type| {
                let element_type    = self.value_for_enum(DbEnumType::VectorElement, Some(element_type)).unwrap().vector_element().unwrap();

                Some(element_type)
            })
    }

    ///
    /// Queries the brush points associated with a vector element
    /// 
    fn query_vector_element_brush_points(&mut self, element_id: i64) -> Result<Vec<BrushPoint>, SqliteAnimationError> {
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

    ///
    /// Queries IDs of the attached elements for a particular item
    ///
    fn query_attached_elements(&mut self, element_id: i64) -> Result<Vec<i64>, SqliteAnimationError> {
        Ok(self.query_map(FloStatement::SelectAttachmentsForElementId, &[&element_id], |row| row.get(0))?
            .filter_map(|row| row.ok())
            .collect())
    }

    ///
    /// Queries a path element
    ///
    fn query_path_element(&mut self, element_id: i64) -> Result<Option<PathElementEntry>, SqliteAnimationError> {
        // The path ID is retrieved via the path element
        let path_id = self.query_row(FloStatement::SelectPathElement, &[&element_id], 
            |element| element.get(0))?;

        // Look for the brush ID and brush properties ID in the attached elements
        // (This slightly weird way of handling attachments is because back in the v2 format these were part of the path element as arbitrary attached elements were not supported)
        let attached                = self.query_attached_elements(element_id)?;
        let mut brush_id            = None;
        let mut brush_properties_id = None;

        for attach_id in attached {
            // Fetch the type of the attachment
            let attachment_type = self.query_vector_element_type_from_element_id(attach_id)?;

            if attachment_type == Some(VectorElementType::BrushProperties) {
                brush_properties_id = Some(attach_id);
            } else if attachment_type == Some(VectorElementType::BrushDefinition) {
                brush_id = Some(attach_id);
            }
        }

        // Generate a path element entry if all the attachments are intact
        if let (Some(brush_id), Some(brush_properties_id)) = (brush_id, brush_properties_id) {
            Ok(Some(PathElementEntry {
                element_id:             element_id,
                path_id:                path_id,
                brush_id:               brush_id,
                brush_properties_id:    brush_properties_id
            }))
        } else {
            // Path missing properties
            Ok(None)
        }
    }

    ///
    /// Queries the path components associated with a vector element
    ///
    fn query_path_components(&mut self, path_id: i64) -> Result<Vec<PathComponent>, SqliteAnimationError> {
        // Request the points and types. 'Close' types have no coordinates associated with them, so they can have null x, y coordinates
        let mut points = self.query_map(FloStatement::SelectPathPointsWithTypes, &[&path_id], 
            |row| -> (Option<f64>, Option<f64>, i64) { (row.get(0), row.get(1), row.get(2)) })?
            .map(|row| row.unwrap())
            .map(|(x, y, point_type)| (x.and_then(|x| y.map(|y| PathPoint::new(x as f32, y as f32))), point_type))
            .map(|(point, point_type)| (point, self.value_for_enum(DbEnumType::PathPoint, Some(point_type)).unwrap()));

        // Iterate through the points to generate the components
        let mut components = vec![];

        while let Some((point, point_type)) = points.next() {
            match point_type {
                DbEnum::PathPoint(PathPointType::MoveTo) => { 
                    components.push(PathComponent::Move(point.unwrap()));
                },

                DbEnum::PathPoint(PathPointType::LineTo) => { 
                    components.push(PathComponent::Line(point.unwrap()));
                },

                DbEnum::PathPoint(PathPointType::ControlPoint) => { 
                    let cp1                 = point;
                    let (cp2, _cp2_type)    = points.next().unwrap();
                    let (tgt, _tgt_type)    = points.next().unwrap();

                    components.push(PathComponent::Bezier(tgt.unwrap(), cp1.unwrap(), cp2.unwrap()));
                },

                DbEnum::PathPoint(PathPointType::BezierTo) => { },

                DbEnum::PathPoint(PathPointType::Close) => {
                    components.push(PathComponent::Close);
                },

                _ => unimplemented!("Enum is not a point type")
            }
        }

        Ok(components)
    }

    ///
    /// Queries the motion associated with a particular motion ID
    /// 
    fn query_motion(&mut self, motion_id: i64) -> Result<Option<MotionEntry>, SqliteAnimationError> {
        let result = self.query_map(FloStatement::SelectMotion, &[&motion_id], |row| (row.get(0), row.get(1), row.get(2)))?
            .map(|row| row.unwrap())
            .map(|(motion_type, x, y): (i64, Option<f64>, Option<f64>)| {
                let motion_type = self.value_for_enum(DbEnumType::MotionType, Some(motion_type));
                let motion_type = motion_type.unwrap().motion_type().unwrap();
                let origin      = x.and_then(|x| y.map(move |y| (x as f32, y as f32)));

                MotionEntry { motion_type, origin }
            })
            .nth(0);

        Ok(result)
    }

    ///
    /// Queries the time points attached to a motion
    /// 
    fn query_motion_timepoints(&mut self, motion_id: i64, path_type: MotionPathType) -> Result<Vec<TimePointEntry>, SqliteAnimationError> {
        let path_type = self.enum_value(DbEnum::MotionPathType(path_type));

        let result = self.query_map(FloStatement::SelectMotionTimePoints, &[&motion_id, &path_type],
            |row| (row.get(0), row.get(1), row.get(2)))?
            .map(|row_with_error| row_with_error.unwrap())
            .map(|(x, y, millis): (f64, f64, f64)| {
                let (x, y, millis) = (x as f32, y as f32, millis as f32);
                TimePointEntry { 
                    x:              x, 
                    y:              y, 
                    milliseconds:   millis 
                }
            })
            .collect();
        
        Ok(result)
    }
}

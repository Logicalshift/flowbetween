use super::*;

use flo_animation::*;

impl FloSqlite {
    ///
    /// Executes a particular database update
    /// 
    fn execute_update(&mut self, update: &DatabaseUpdate) -> Result<(), SqliteAnimationError> {
        use self::DatabaseUpdate::*;

        match update {
            Pop                                                             => {
                #[cfg(test)]
                {
                    if self.stack.len() == 0 {
                        panic!("Popping on empty stack");
                    }
                }

                self.stack.pop(); 
            },

            Duplicate                                                       => {
                let on_top = *(self.stack.last().unwrap());
                self.stack.push(on_top);
            },

            UpdateCanvasSize(width, height)                                 => {
                let mut update_size = Self::prepare(&self.sqlite, FloStatement::UpdateAnimationSize)?;
                update_size.execute::<&[&dyn ToSql]>(&[&width, &height, &self.animation_id])?;
            },

            PushEditType(edit_log_type)                                     => {
                let edit_log_type   = self.enum_value(DbEnum::EditLog(*edit_log_type));
                let edit_log_id     = Self::prepare(&self.sqlite, FloStatement::InsertEditType)?.insert::<&[&dyn ToSql]>(&[&edit_log_type])?;
                self.stack.push(edit_log_id);
            },

            PopEditLogSetSize(width, height)                                => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_size    = Self::prepare(&self.sqlite, FloStatement::InsertELSetSize)?;
                set_size.insert::<&[&dyn ToSql]>(&[&edit_log_id, &(*width as f64), &(*height as f64)])?;
            },

            PushEditLogLayer(layer_id)                                      => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_layer   = Self::prepare(&self.sqlite, FloStatement::InsertELLayer)?;
                set_layer.insert::<&[&dyn ToSql]>(&[&edit_log_id, &(*layer_id as i64)])?;
                self.stack.push(edit_log_id);
            },

            PushEditLogWhen(when)                                           => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_when    = Self::prepare(&self.sqlite, FloStatement::InsertELWhen)?;
                set_when.insert::<&[&dyn ToSql]>(&[&edit_log_id, &Self::get_micros(&when)])?;
                self.stack.push(edit_log_id);
            },

            PopEditLogBrush(drawing_style)                                  => {
                let brush_id        = self.stack.pop().unwrap();
                let edit_log_id     = self.stack.pop().unwrap();
                let drawing_style   = self.enum_value(DbEnum::DrawingStyle(*drawing_style));
                let mut set_brush   = Self::prepare(&self.sqlite, FloStatement::InsertELBrush)?;
                set_brush.insert::<&[&dyn ToSql]>(&[&edit_log_id, &drawing_style, &brush_id])?;
            },

            PopEditLogString(index, string)                                 => {
                let edit_log_id             = self.stack.pop().unwrap();
                let mut insert_edit_string  = Self::prepare(&self.sqlite, FloStatement::InsertELString)?;
                let index                   = *index as i64;
                insert_edit_string.insert::<&[&dyn ToSql]>(&[&edit_log_id, &index, string])?;
            },

            PushEditLogInt(index, value)                                    => {
                let edit_log_id             = self.stack.last().unwrap();
                let mut insert_edit_int     = Self::prepare(&self.sqlite, FloStatement::InsertELInt)?;
                let index                   = *index as i64;
                insert_edit_int.insert::<&[&dyn ToSql]>(&[&edit_log_id, &index, value])?;
            },

            PushEditLogFloat(index, value)                                  => {
                let edit_log_id             = self.stack.last().unwrap();
                let mut insert_edit_int     = Self::prepare(&self.sqlite, FloStatement::InsertELFloat)?;
                let index                   = *index as i64;
                insert_edit_int.insert::<&[&dyn ToSql]>(&[&edit_log_id, &index, value])?;
            },

            PopEditLogBrushProperties                                       => {
                let brush_props_id      = self.stack.pop().unwrap();
                let edit_log_id         = self.stack.pop().unwrap();
                let mut set_brush_props = Self::prepare(&self.sqlite, FloStatement::InsertELBrushProperties)?;
                set_brush_props.insert::<&[&dyn ToSql]>(&[&edit_log_id, &brush_props_id])?;
            },

            PushEditLogElementId(index, element_id)                          => {
                let edit_log_id         = self.stack.last().unwrap();
                let mut add_element_id  = Self::prepare(&self.sqlite, FloStatement::InsertELElementId)?;
                let index               = *index as i64;
                
                add_element_id.insert::<&[&dyn ToSql]>(&[edit_log_id, &index, element_id])?;
            },

            PushRawPoints(points)                                           => {
                let edit_log_id         = self.stack.last().unwrap();
                let mut add_raw_point   = Self::prepare(&self.sqlite, FloStatement::InsertELRawPoints)?;
                let mut point_bytes     = vec![];

                write_raw_points(&mut point_bytes, &*points).unwrap();
                add_raw_point.insert::<&[&dyn ToSql]>(&[edit_log_id, &point_bytes])?;
            },

            PushEditLogMotionOrigin(x, y) => {
                let (x, y)          = (*x as f64, *y as f64);
                let edit_log_id     = self.stack.last().unwrap();
                let mut add_origin  = Self::prepare(&self.sqlite, FloStatement::InsertELMotionOrigin)?;
                
                add_origin.insert::<&[&dyn ToSql]>(&[edit_log_id, &x, &y])?;
            },

            PushEditLogMotionType(motion_type) => {
                let motion_type     = self.enum_value(DbEnum::MotionType(*motion_type));
                let edit_log_id     = self.stack.last().unwrap();
                let mut add_type    = Self::prepare(&self.sqlite, FloStatement::InsertELMotionType)?;

                add_type.insert::<&[&dyn ToSql]>(&[edit_log_id, &motion_type])?;
            },

            PushEditLogMotionElement(attach_element) => {
                let edit_log_id     = self.stack.last().unwrap();
                let mut add_type    = Self::prepare(&self.sqlite, FloStatement::InsertELMotionElement)?;

                add_type.insert::<&[&dyn ToSql]>(&[edit_log_id, &attach_element])?;
            },

            PushEditLogMotionPath(num_points) => {
                // Collect the IDs of the points
                let mut point_ids = vec![];
                for _index in 0..*num_points {
                    point_ids.push(self.stack.pop().unwrap_or(-1));
                }

                // The edit log ID is found underneath the stack of points
                let edit_log_id = self.stack.last().unwrap();

                // Prepare the insertion statement
                let mut add_point = Self::prepare(&self.sqlite, FloStatement::InsertELMotionTimePoint)?;

                // Insert each of the points in turn
                for index in 0..*num_points {
                    let point_index = ((num_points-1)-index) as i64;
                    add_point.insert::<&[&dyn ToSql]>(&[edit_log_id, &point_index, &point_ids[index]])?;
                }
            },

            PushEditLogPath => {
                let mut insert_el_path  = Self::prepare(&self.sqlite, FloStatement::InsertELPath)?;
                let path_id             = self.stack.pop().unwrap_or(-1);
                let edit_log_id         = self.stack.pop().unwrap_or(-1);

                insert_el_path.insert::<&[&dyn ToSql]>(&[&edit_log_id, &path_id])?;

                self.stack.push(edit_log_id);
            },

            PushPath(points) => {
                let mut insert_point    = Self::prepare(&self.sqlite, FloStatement::InsertPathPoint)?;
                let mut insert_path     = Self::prepare(&self.sqlite, FloStatement::InsertPath)?;
                let path_id             = insert_path.insert::<&[&dyn ToSql]>(&[])?;
                let path_id             = path_id as i64;

                for point_index in 0..(points.len()) {
                    let (x, y)      = points[point_index];
                    let (x, y)      = (x as f64, y as f64);
                    let point_index = point_index as i64;
                    insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &x, &y])?;
                }

                self.stack.push(path_id);
            },

            PushPathComponents(components) => {
                let point_move_to       = self.enum_value(DbEnum::PathPoint(PathPointType::MoveTo));
                let point_line_to       = self.enum_value(DbEnum::PathPoint(PathPointType::LineTo));
                let point_control_point = self.enum_value(DbEnum::PathPoint(PathPointType::ControlPoint));
                let point_bezier_to     = self.enum_value(DbEnum::PathPoint(PathPointType::BezierTo));
                let point_close         = self.enum_value(DbEnum::PathPoint(PathPointType::Close));

                let mut insert_path     = Self::prepare(&self.sqlite, FloStatement::InsertPath)?;
                let mut insert_point    = Self::prepare(&self.sqlite, FloStatement::InsertPathPoint)?;
                let mut insert_type     = Self::prepare(&self.sqlite, FloStatement::InsertPathPointType)?;

                // Create the path
                let path_id             = insert_path.insert::<&[&dyn ToSql]>(&[])?;
                let path_id             = path_id as i64;

                // Insert the components
                let mut point_index = 0;
                for component in components.iter() {
                    use self::PathComponent::*;

                    match component {
                        Move(point) => { 
                            let (x, y) = point.position;
                            let (x, y) = (x as f64, y as f64);
                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &x, &y])?;
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &point_move_to])?;
                        },

                        Line(point) => {
                            let (x, y) = point.position;
                            let (x, y) = (x as f64, y as f64);
                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &x, &y])?;
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &point_line_to])?;
                        },
                        
                        Bezier(target, cp1, cp2) => {
                            let (tx, ty)        = target.position;
                            let (cp1x, cp1y)    = cp1.position;
                            let (cp2x, cp2y)    = cp2.position;

                            let (tx, ty)        = (tx as f64, ty as f64);    
                            let (cp1x, cp1y)    = (cp1x as f64, cp1y as f64);
                            let (cp2x, cp2y)    = (cp2x as f64, cp2y as f64);

                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &cp1x, &cp1y])?;
                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index+1), &cp2x, &cp2y])?;
                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index+2), &tx, &ty])?;

                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &point_control_point])?;
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index+1), &point_control_point])?;
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index+2), &point_bezier_to])?;

                            point_index += 2;
                        },

                        Close => {
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index), &point_close])?;
                        }
                    }

                    point_index += 1;
                }

                // Final stack is just the path ID
                self.stack.push(path_id);
            },

            PopRemovePathPoints(point_range)                                => {
                let path_id             = self.stack.pop().unwrap();

                let mut delete_points   = Self::prepare(&self.sqlite, FloStatement::DeletePathPointRange)?;
                let mut delete_types    = Self::prepare(&self.sqlite, FloStatement::DeletePathPointTypeRange)?;
                let mut update_points   = Self::prepare(&self.sqlite, FloStatement::UpdatePathPointIndicesAfter)?;
                let mut update_types    = Self::prepare(&self.sqlite, FloStatement::UpdatePathPointTypeIndicesAfter)?;

                let lower_point         = point_range.start as i64;
                let upper_point         = point_range.end as i64;

                delete_points.execute(&[&path_id, &lower_point, &upper_point])?;
                delete_types.execute(&[&path_id, &lower_point, &upper_point])?;
                update_points.execute(&[&(lower_point-upper_point), &path_id, &lower_point])?;
                update_types.execute(&[&(lower_point-upper_point), &path_id, &lower_point])?;
            },

            PopInsertPathComponents(initial_point_index, components)        => {
                let path_id             = self.stack.pop().unwrap();

                let point_move_to       = self.enum_value(DbEnum::PathPoint(PathPointType::MoveTo));
                let point_line_to       = self.enum_value(DbEnum::PathPoint(PathPointType::LineTo));
                let point_control_point = self.enum_value(DbEnum::PathPoint(PathPointType::ControlPoint));
                let point_bezier_to     = self.enum_value(DbEnum::PathPoint(PathPointType::BezierTo));
                let point_close         = self.enum_value(DbEnum::PathPoint(PathPointType::Close));

                let mut update_points   = Self::prepare(&self.sqlite, FloStatement::UpdatePathPointIndicesAfter)?;
                let mut update_types    = Self::prepare(&self.sqlite, FloStatement::UpdatePathPointTypeIndicesAfter)?;
                let mut insert_point    = Self::prepare(&self.sqlite, FloStatement::InsertPathPoint)?;
                let mut insert_type     = Self::prepare(&self.sqlite, FloStatement::InsertPathPointType)?;

                let initial_point_index = *initial_point_index as i64;

                // Count the points in the path
                let total_num_points = components.iter().map(|component| component.num_points()).sum::<usize>() as i64;

                // Update the point indexes in this range
                update_points.execute(&[&total_num_points, &path_id, &initial_point_index])?;
                update_types.execute(&[&total_num_points, &path_id, &initial_point_index])?;

                // Insert the new points
                // TODO: dedupe with PushPathComponents
                let mut point_index = initial_point_index;
                for component in components.iter() {
                    use self::PathComponent::*;

                    match component {
                        Move(point) => { 
                            let (x, y) = point.position;
                            let (x, y) = (x as f64, y as f64);
                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &x, &y])?;
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &point_move_to])?;
                        },

                        Line(point) => {
                            let (x, y) = point.position;
                            let (x, y) = (x as f64, y as f64);
                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &x, &y])?;
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &point_line_to])?;
                        },
                        
                        Bezier(target, cp1, cp2) => {
                            let (tx, ty)        = target.position;
                            let (cp1x, cp1y)    = cp1.position;
                            let (cp2x, cp2y)    = cp2.position;

                            let (tx, ty)        = (tx as f64, ty as f64);    
                            let (cp1x, cp1y)    = (cp1x as f64, cp1y as f64);
                            let (cp2x, cp2y)    = (cp2x as f64, cp2y as f64);

                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &cp1x, &cp1y])?;
                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index+1), &cp2x, &cp2y])?;
                            insert_point.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index+2), &tx, &ty])?;

                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &point_index, &point_control_point])?;
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index+1), &point_control_point])?;
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index+2), &point_bezier_to])?;

                            point_index += 2;
                        },

                        Close => {
                            insert_type.insert::<&[&dyn ToSql]>(&[&path_id, &(point_index), &point_close])?;
                        }
                    }

                    point_index += 1;
                }
            },

            PushTimePoint(x, y, millis) => {
                let (x, y, millis)  = (*x as f64, *y as f64, *millis as f64);
                let mut add_point   = Self::prepare(&self.sqlite, FloStatement::InsertTimePoint)?;
                let point_id        = add_point.insert::<&[&dyn ToSql]>(&[&x, &y, &millis])?;
                self.stack.push(point_id);
            },

            PushBrushType(brush_type)                                       => {
                let brush_type              = self.enum_value(DbEnum::BrushDefinition(*brush_type));
                let mut insert_brush_type   = Self::prepare(&self.sqlite, FloStatement::InsertBrushType)?;
                let brush_id                = insert_brush_type.insert::<&[&dyn ToSql]>(&[&brush_type])?;
                self.stack.push(brush_id);
            },

            PushInkBrush(min_width, max_width, scale_up_distance)           => {
                let brush_id                = self.stack.last().unwrap();
                let mut insert_ink_brush    = Self::prepare(&self.sqlite, FloStatement::InsertInkBrush)?;
                insert_ink_brush.insert::<&[&dyn ToSql]>(&[brush_id, &(*min_width as f64), &(*max_width as f64), &(*scale_up_distance as f64)])?;
            },

            PushBrushProperties(size, opacity)                              => {
                let color_id                    = self.stack.pop().unwrap();
                let mut insert_brush_properties = Self::prepare(&self.sqlite, FloStatement::InsertBrushProperties)?;
                let brush_props_id              = insert_brush_properties.insert::<&[&dyn ToSql]>(&[&(*size as f64), &(*opacity as f64), &color_id])?;
                self.stack.push(brush_props_id);
            },

            PushColorType(color_type)                                       => {
                let color_type              = self.enum_value(DbEnum::Color(*color_type));
                let mut insert_color_type   = Self::prepare(&self.sqlite, FloStatement::InsertColorType)?;
                let color_id                = insert_color_type.insert::<&[&dyn ToSql]>(&[&color_type])?;
                self.stack.push(color_id);
            },

            PushRgb(r, g, b)                                                => {
                let color_id        = self.stack.last().unwrap();
                let mut insert_rgb  = Self::prepare(&self.sqlite, FloStatement::InsertRgb)?;
                insert_rgb.insert::<&[&dyn ToSql]>(&[color_id, &(*r as f64), &(*g as f64), &(*b as f64)])?;
            },

            PushHsluv(h, s, l)                                              => {
                let color_id            = self.stack.last().unwrap();
                let mut insert_hsluv    = Self::prepare(&self.sqlite, FloStatement::InsertHsluv)?;
                insert_hsluv.insert::<&[&dyn ToSql]>(&[color_id, &(*h as f64), &(*s as f64), &(*l as f64)])?;
            },

            PopDeleteLayer                                                  => {
                let layer_id            = self.stack.pop().unwrap();
                let mut delete_layer    = Self::prepare(&self.sqlite, FloStatement::DeleteLayer)?;
                delete_layer.execute::<&[&dyn ToSql]>(&[&layer_id])?;
            },

            PushLayerType(layer_type)                                       => {
                let layer_type              = self.enum_value(DbEnum::Layer(*layer_type));
                let mut insert_layer_type   = Self::prepare(&self.sqlite, FloStatement::InsertLayerType)?;
                let layer_id                = insert_layer_type.insert::<&[&dyn ToSql]>(&[&layer_type])?;
                self.stack.push(layer_id);
            },

            PushAssignLayer(assigned_id)                                    => {
                let layer_id                = self.stack.last().unwrap();
                let mut insert_assign_layer = Self::prepare(&self.sqlite, FloStatement::InsertAssignLayer)?;
                insert_assign_layer.insert::<&[&dyn ToSql]>(&[&self.animation_id, layer_id, &(*assigned_id as i64)])?;
            },

            PushLayerId(layer_id)                                           => {
                self.stack.push(*layer_id);
            },

            PopLayerName(name)                                              => {
                let layer_id                    = self.stack.pop().unwrap();
                let mut insert_or_replace_name  = Self::prepare(&self.sqlite, FloStatement::InsertOrReplaceLayerName)?;
                insert_or_replace_name.insert::<&[&dyn ToSql]>(&[&layer_id, name])?;
            },

            PushLayerForAssignedId(assigned_id)                             => {
                let mut select_layer_id = Self::prepare(&self.sqlite, FloStatement::SelectLayerId)?;
                let layer_id            = select_layer_id.query_row(&[&self.animation_id, &(*assigned_id as i64)], |row| row.get(0))?;
                self.stack.push(layer_id);
            },

            PopAddKeyFrame(when)                                            => {
                let layer_id                = self.stack.pop().unwrap();
                let mut insert_key_frame    = Self::prepare(&self.sqlite, FloStatement::InsertKeyFrame)?;
                insert_key_frame.insert::<&[&dyn ToSql]>(&[&layer_id, &Self::get_micros(&when)])?;
            },

            PopRemoveKeyFrame(when)                                         => {
                let layer_id                = self.stack.pop().unwrap();
                let mut delete_key_frame    = Self::prepare(&self.sqlite, FloStatement::DeleteKeyFrame)?;
                delete_key_frame.execute::<&[&dyn ToSql]>(&[&layer_id, &Self::get_micros(&when)])?;
            },

            PopStoreLayerCache(when, cache_type, canvas_data)               => {
                let layer_id                    = self.stack.pop().unwrap();
                let when                        = Self::get_micros(&when);
                let cache_type                  = self.enum_value(DbEnum::CacheType(*cache_type));

                let mut delete_layer_cache      = Self::prepare(&self.sqlite, FloStatement::DeleteLayerCache)?;
                let mut insert_layer_drawing    = Self::prepare(&self.sqlite, FloStatement::InsertNewCachedDrawing)?;
                let mut insert_layer_cache      = Self::prepare(&self.sqlite, FloStatement::InsertOrReplaceLayerCache)?;

                delete_layer_cache.execute::<&[&dyn ToSql]>(&[&cache_type, &layer_id, &when])?;
                let cache_id = insert_layer_drawing.insert(&[&canvas_data])?;
                insert_layer_cache.execute::<&[&dyn ToSql]>(&[&cache_type, &layer_id, &when, &cache_id])?;
            },

            PopDeleteLayerCache(when, cache_type)                        => {
                let layer_id                    = self.stack.pop().unwrap();
                let when                        = Self::get_micros(&when);
                let cache_type                  = self.enum_value(DbEnum::CacheType(*cache_type));

                let mut delete_layer_cache      = Self::prepare(&self.sqlite, FloStatement::DeleteLayerCache)?;

                delete_layer_cache.execute::<&[&dyn ToSql]>(&[&cache_type, &layer_id, &when])?;
            },

            PushNearestKeyFrame(when)                                       => {
                let layer_id                        = self.stack.pop().unwrap();
                let mut select_nearest_keyframe     = Self::prepare(&self.sqlite, FloStatement::SelectNearestKeyFrame)?;
                let (keyframe_id, start_micros)     = select_nearest_keyframe.query_row(&[&layer_id, &(Self::get_micros(&when))], |row| Ok((row.get(0)?, row.get(1)?)))?;
                self.stack.push(start_micros);
                self.stack.push(keyframe_id);
            },

            PushVectorElementType(element_type)                             => {
                // Create the element
                let element_type                    = self.enum_value(DbEnum::VectorElement(*element_type));
                let mut insert_vector_element_type  = Self::prepare(&self.sqlite, FloStatement::InsertVectorElementType)?;
                let element_id                      = insert_vector_element_type.insert::<&[&dyn ToSql]>(&[&&element_type])?;
                self.stack.push(element_id);
            },

            PushVectorElementTime(when)                                     => {
                // Set when the element is
                let element_id                      = self.stack.pop().unwrap();
                let keyframe_id                     = self.stack.pop().unwrap();
                let start_micros                    = self.stack.pop().unwrap();
                let mut insert_vector_element_time  = Self::prepare(&self.sqlite, FloStatement::InsertOrReplaceVectorElementTime)?;
                let when                            = Self::get_micros(&when) - start_micros;
                let element_id                      = insert_vector_element_time.insert::<&[&dyn ToSql]>(&[&element_id, &keyframe_id, &when])?;
                self.stack.push(start_micros);
                self.stack.push(keyframe_id);
                self.stack.push(element_id);

                // Set it to the highest z-index
                let mut select_max_z_index          = Self::prepare(&self.sqlite, FloStatement::SelectMaxZIndexForKeyFrame)?;
                let max_z_index                     = select_max_z_index.query_row(&[&keyframe_id], |row| row.get::<_, i64>(0))?;

                let new_z_index                     = max_z_index + 1;
                let mut insert_z_index              = Self::prepare(&self.sqlite, FloStatement::InsertOrReplaceZIndex)?;
                insert_z_index.insert(&[&element_id, &keyframe_id, &new_z_index])?;
            },

            PushElementAssignId(assigned_id)                                => {
                let element_id                      = self.stack.last().unwrap();
                let mut insert_element_assigned_id  = Self::prepare(&self.sqlite, FloStatement::InsertElementAssignedId)?;
                insert_element_assigned_id.insert::<&[&dyn ToSql]>(&[element_id, &assigned_id])?;
            },

            PushElementIdForAssignedId(assigned_id)                         => {
                let mut element_id_for_assigned_id  = Self::prepare(&self.sqlite, FloStatement::SelectElementIdForAssignedId)?;
                let element_id                      = element_id_for_assigned_id.query_row(&[&assigned_id], |row| row.get(0))?;
                self.stack.push(element_id);
            },

            PushAttachElements(num_to_attach)                               => {
                let mut attach_element_ids      = vec![];
                for _ in 0..*num_to_attach {
                    attach_element_ids.push(self.stack.pop().unwrap());
                }

                let attach_to_element_id        = self.stack.last().unwrap();
                let mut insert_attach_element   = Self::prepare(&self.sqlite, FloStatement::InsertAttachElement)?;

                for attach_element_id in attach_element_ids {
                    insert_attach_element.insert::<&[&dyn ToSql]>(&[&attach_to_element_id, &attach_element_id])?;
                }
            },

            PushDetachElements(num_to_detach)                               => {
                let mut detach_element_ids      = vec![];
                for _ in 0..*num_to_detach {
                    detach_element_ids.push(self.stack.pop().unwrap());
                }

                let detach_from_element_id          = self.stack.last().unwrap();
                let mut delete_element_attachment   = Self::prepare(&self.sqlite, FloStatement::DeleteElementAttachment)?;

                for detach_element_id in detach_element_ids {
                    delete_element_attachment.execute::<&[&dyn ToSql]>(&[&detach_from_element_id, &detach_element_id])?;
                }
            },

            PushKeyFrameIdForElementId                                      => {
                let element_id                      = self.stack.pop().unwrap();
                let mut key_frame_for_element_id    = Self::prepare(&self.sqlite, FloStatement::SelectElementKeyFrame)?;

                let key_frame_id                    = key_frame_for_element_id.query_row(&[&element_id], |row| row.get(0))?;
                self.stack.push(key_frame_id);
                self.stack.push(element_id);
            },

            PushPathIdForElementId                                          => {
                let element_id                      = self.stack.pop().unwrap();
                let mut path_for_element_id         = Self::prepare(&self.sqlite, FloStatement::SelectPathElement)?;

                let path_id                         = path_for_element_id.query_row(&[&element_id], |row| Ok(row.get(0)?))?;
                self.stack.push(element_id);
                self.stack.push(path_id);
            }

            PopVectorBrushElement(drawing_style)                            => {
                let brush_id                            = self.stack.pop().unwrap();
                let element_id                          = self.stack.pop().unwrap();
                let drawing_style                       = self.enum_value(DbEnum::DrawingStyle(*drawing_style));
                let mut insert_brush_definition_element = Self::prepare(&self.sqlite, FloStatement::InsertBrushDefinitionElement)?;
                insert_brush_definition_element.insert::<&[&dyn ToSql]>(&[&element_id, &brush_id, &drawing_style])?;
            },

            PopVectorBrushPropertiesElement                                 => {
                let brush_props_id                  = self.stack.pop().unwrap();
                let element_id                      = self.stack.pop().unwrap();
                let mut insert_brush_props_element  = Self::prepare(&self.sqlite, FloStatement::InsertBrushPropertiesElement)?;
                insert_brush_props_element.insert::<&[&dyn ToSql]>(&[&element_id, &brush_props_id])?;
            },

            PopVectorPathElement                                            => {
                let path_id                     = self.stack.pop().unwrap();
                let brush_properties_id         = self.stack.pop().unwrap();
                let brush_id                    = self.stack.pop().unwrap();
                let element_id                  = self.stack.pop().unwrap();
                let mut insert_path_element     = Self::prepare(&self.sqlite, FloStatement::InsertPathElement)?;
                let mut insert_attach_element   = Self::prepare(&self.sqlite, FloStatement::InsertAttachElement)?;
                insert_path_element.insert::<&[&dyn ToSql]>(&[&element_id, &path_id])?;
                insert_attach_element.insert::<&[&dyn ToSql]>(&[&element_id, &brush_id])?;
                insert_attach_element.insert::<&[&dyn ToSql]>(&[&element_id, &brush_properties_id])?;
            },

            PopBrushPoints(points)                                          => {
                let element_id              = self.stack.pop().unwrap();
                let mut insert_brush_point  = Self::prepare(&self.sqlite, FloStatement::InsertBrushPoint)?;

                let num_points = points.len();
                for (point, index) in points.iter().zip((0..num_points).into_iter()) {
                    insert_brush_point.insert::<&[&dyn ToSql]>(&[
                        &element_id, &(index as i64),
                        &(point.cp1.0 as f64), &(point.cp1.1 as f64),
                        &(point.cp2.0 as f64), &(point.cp2.1 as f64),
                        &(point.position.0 as f64), &(point.position.1 as f64),
                        &(point.width as f64)
                    ])?;
                }
            },

            UpdateBrushPointCoords(points)                                  => {
                let element_id              = self.stack.pop().unwrap();
                let mut update_brush_point  = Self::prepare(&self.sqlite, FloStatement::UpdateBrushPoint)?;

                for (index, ((x1, y1), (x2, y2), (x3, y3))) in points.iter().enumerate() {
                    let (x1, y1, x2, y2, x3, y3) = (*x1 as f64, *y1 as f64, *x2 as f64, *y2 as f64, *x3 as f64, *y3 as f64);

                    update_brush_point.execute::<&[&dyn ToSql]>(&[&x1, &y1, &x2, &y2, &x3, &y3, &element_id, &(index as i64)])?;
                }
            },

            UpdatePathPointCoords(points)                                   => {
                let path_id                 = self.stack.pop().unwrap();
                let mut update_path_point   = Self::prepare(&self.sqlite, FloStatement::UpdatePathPoint)?;

                for (index, (x, y)) in points.iter().enumerate() {
                    let (x, y)  = (*x as f64, *y as f64);
                    let index   = index as i64;
                    update_path_point.execute::<&[&dyn ToSql]>(&[&x, &y, &path_id, &index])?;
                }
            },

            PopVectorElementMove(move_direction)                            => {
                // The stack contains the element ID and the keyframe ID
                let element_id          = self.stack.pop().unwrap();
                let keyframe_id         = self.stack.pop().unwrap();

                // Fetch the current position
                let mut select_current_zindex   = Self::prepare(&self.sqlite, FloStatement::SelectZIndexForElement)?;
                let current_zindex              = select_current_zindex.query_row(&[&element_id], |row| row.get::<_, i64>(0))?;

                // Need to work out the target z-index
                let target_z_index      = match move_direction {
                    DbElementMove::ToBottom     => { 0 },
                    DbElementMove::ToTop        => { 
                        let mut select_max_z_index = Self::prepare(&self.sqlite, FloStatement::SelectMaxZIndexForKeyFrame)?;
                        select_max_z_index.query_row(&[&keyframe_id], |row| row.get::<_, i64>(0))? + 1
                    },
                    DbElementMove::Up           => {
                        let mut select_next_z_index = Self::prepare(&self.sqlite, FloStatement::SelectZIndexAfterZIndexForKeyFrame)?;
                        select_next_z_index.query_row(&[&keyframe_id, &current_zindex], |row| row.get::<_, i64>(0))? + 1
                    },
                    DbElementMove::Down         => {
                        let mut select_previous_z_index = Self::prepare(&self.sqlite, FloStatement::SelectZIndexBeforeZIndexForKeyFrame)?;
                        select_previous_z_index.query_row(&[&keyframe_id, &current_zindex], |row| row.get(0))?
                    }
                };

                // Remove from the current position in the frame
                let mut delete_element_zindex   = Self::prepare(&self.sqlite, FloStatement::DeleteElementZIndex)?;
                let mut move_z_index_down       = Self::prepare(&self.sqlite, FloStatement::UpdateMoveZIndexDownwards)?;
                delete_element_zindex.execute(&[&element_id])?;
                move_z_index_down.execute(&[&keyframe_id, &current_zindex])?;

                let target_z_index = if target_z_index >= current_zindex {
                    target_z_index - 1
                } else {
                    target_z_index
                };

                // Create space around the target z-index
                let mut move_z_index_up         = Self::prepare(&self.sqlite, FloStatement::UpdateMoveZIndexUpwards)?;
                move_z_index_up.execute(&[&keyframe_id, &target_z_index])?;

                // Update the z-index of the element
                let mut insert_replace_zindex   = Self::prepare(&self.sqlite, FloStatement::InsertOrReplaceZIndex)?;
                insert_replace_zindex.insert(&[&element_id, &keyframe_id, &target_z_index])?;
            },

            CreateMotion(motion_id)                                         => {
                let motion_type         = self.enum_value(DbEnum::MotionType(MotionType::None));
                let mut insert_motion   = Self::prepare(&self.sqlite, FloStatement::InsertMotion)?;

                insert_motion.insert::<&[&dyn ToSql]>(&[&motion_id, &motion_type])?;
            },

            SetMotionType(motion_id, motion_type)                           => {
                let motion_type         = self.enum_value(DbEnum::MotionType(*motion_type));
                let mut update_motion   = Self::prepare(&self.sqlite, FloStatement::UpdateMotionType)?;

                update_motion.insert::<&[&dyn ToSql]>(&[&motion_type, &motion_id])?;
            },

            SetMotionOrigin(motion_id, x, y)                                => {
                let mut set_origin  = Self::prepare(&self.sqlite, FloStatement::InsertOrReplaceMotionOrigin)?;
                let (x, y)          = (*x as f64, *y as f64);

                set_origin.insert::<&[&dyn ToSql]>(&[&motion_id, &x, &y])?;
            },

            SetMotionPath(motion_id, path_type, num_points)                 => {
                let path_type           = self.enum_value(DbEnum::MotionPathType(*path_type));
                let mut delete_path     = Self::prepare(&self.sqlite, FloStatement::DeleteMotionPoints)?;
                let mut insert_point    = Self::prepare(&self.sqlite, FloStatement::InsertMotionPathPoint)?;

                // Remove the existing path of this type from the motion
                delete_path.execute::<&[&dyn ToSql]>(&[&motion_id, &path_type])?;

                // Collect the IDs of the points
                let mut point_ids = vec![];
                for _index in 0..*num_points {
                    point_ids.push(self.stack.pop().unwrap_or(-1));
                }

                // Insert these points
                for index in 0..*num_points {
                    let point_index = ((num_points-1)-index) as i64;
                    insert_point.insert::<&[&dyn ToSql]>(&[&motion_id, &path_type, &point_index, &point_ids[index]])?;
                }
            },

            DeleteMotion(motion_id)                                         => {
                let mut delete_motion = Self::prepare(&self.sqlite, FloStatement::DeleteMotion)?;
                delete_motion.execute::<&[&dyn ToSql]>(&[&motion_id])?;
            },
        }

        Ok(())
    }

    ///
    /// Performs a set of updates on the database immediately
    /// 
    fn execute_updates_now<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<(), SqliteAnimationError> {
        for update in updates {
            let result = self.execute_update(&update);

            if let Err(failure) = result {
                self.log.log((Level::Error, format!("Update operation `{:?}` failed: `{:?}`", update, failure)));
                return Err(failure);
            }
        }
        Ok(())
    }
}

impl FloStore for FloSqlite {
    ///
    /// Performs a set of updates on the database
    /// 
    fn update<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<(), SqliteAnimationError> {
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
    fn begin_queuing(&mut self) {
        if self.pending.is_none() {
            self.pending = Some(vec![]);
        }
    }

    ///
    /// Executes the update queue
    /// 
    fn execute_queue(&mut self) -> Result<(), SqliteAnimationError> {
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
    fn flush_pending(&mut self) -> Result<(), SqliteAnimationError> {
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
}

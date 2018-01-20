use super::*;

impl FloSqlite {
    ///
    /// Executes a particular database update
    /// 
    fn execute_update(&mut self, update: DatabaseUpdate) -> Result<()> {
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
                Ok(()) 
            },

            UpdateCanvasSize(width, height)                                 => {
                let mut update_size = Self::prepare(&self.sqlite, FloStatement::UpdateAnimationSize)?;
                update_size.execute(&[&width, &height, &self.animation_id])?;
                Ok(())
            },

            PushEditType(edit_log_type)                                     => {
                let edit_log_type   = self.enum_value(DbEnum::EditLog(edit_log_type));
                let edit_log_id     = Self::prepare(&self.sqlite, FloStatement::InsertEditType)?.insert(&[&edit_log_type])?;
                self.stack.push(edit_log_id);
                Ok(())
            },

            PopEditLogSetSize(width, height)                                => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_size    = Self::prepare(&self.sqlite, FloStatement::InsertELSetSize)?;
                set_size.insert(&[&edit_log_id, &(width as f64), &(height as f64)])?;
                Ok(())
            },

            PushEditLogLayer(layer_id)                                      => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_layer   = Self::prepare(&self.sqlite, FloStatement::InsertELLayer)?;
                set_layer.insert(&[&edit_log_id, &(layer_id as i64)])?;
                self.stack.push(edit_log_id);
                Ok(())
            },

            PushEditLogWhen(when)                                           => {
                let edit_log_id     = self.stack.pop().unwrap();
                let mut set_when    = Self::prepare(&self.sqlite, FloStatement::InsertELWhen)?;
                set_when.insert(&[&edit_log_id, &Self::get_micros(&when)])?;
                self.stack.push(edit_log_id);
                Ok(())
            },

            PopEditLogBrush(drawing_style)                                  => {
                let brush_id        = self.stack.pop().unwrap();
                let edit_log_id     = self.stack.pop().unwrap();
                let drawing_style   = self.enum_value(DbEnum::DrawingStyle(drawing_style));
                let mut set_brush   = Self::prepare(&self.sqlite, FloStatement::InsertELBrush)?;
                set_brush.insert(&[&edit_log_id, &drawing_style, &brush_id])?;
                Ok(())
            },

            PopEditLogBrushProperties                                       => {
                let brush_props_id      = self.stack.pop().unwrap();
                let edit_log_id         = self.stack.pop().unwrap();
                let mut set_brush_props = Self::prepare(&self.sqlite, FloStatement::InsertELBrushProperties)?;
                set_brush_props.insert(&[&edit_log_id, &brush_props_id])?;
                Ok(())
            },

            PushRawPoints(points)                                           => {
                let edit_log_id         = self.stack.last().unwrap();
                let mut add_raw_point   = Self::prepare(&self.sqlite, FloStatement::InsertELRawPoint)?;
                let num_points          = points.len();

                for (point, index) in points.iter().zip((0..num_points).into_iter()) {
                    add_raw_point.insert(&[
                        edit_log_id, &(index as i64), 
                        &(point.position.0 as f64), &(point.position.1 as f64),
                        &(point.pressure as f64),
                        &(point.tilt.0 as f64), &(point.tilt.1 as f64)
                    ])?;
                }

                Ok(())                
            },

            PushBrushType(brush_type)                                       => {
                let brush_type              = self.enum_value(DbEnum::BrushDefinition(brush_type));
                let mut insert_brush_type   = Self::prepare(&self.sqlite, FloStatement::InsertBrushType)?;
                let brush_id                = insert_brush_type.insert(&[&brush_type])?;
                self.stack.push(brush_id);
                Ok(())
            },

            PushInkBrush(min_width, max_width, scale_up_distance)           => {
                let brush_id                = self.stack.last().unwrap();
                let mut insert_ink_brush    = Self::prepare(&self.sqlite, FloStatement::InsertInkBrush)?;
                insert_ink_brush.insert(&[brush_id, &(min_width as f64), &(max_width as f64), &(scale_up_distance as f64)])?;
                Ok(())
            },

            PushBrushProperties(size, opacity)                              => {
                let color_id                    = self.stack.pop().unwrap();
                let mut insert_brush_properties = Self::prepare(&self.sqlite, FloStatement::InsertBrushProperties)?;
                let brush_props_id              = insert_brush_properties.insert(&[&(size as f64), &(opacity as f64), &color_id])?;
                self.stack.push(brush_props_id);
                Ok(())
            },

            PushColorType(color_type)                                       => {
                let color_type              = self.enum_value(DbEnum::Color(color_type));
                let mut insert_color_type   = Self::prepare(&self.sqlite, FloStatement::InsertColorType)?;
                let color_id                = insert_color_type.insert(&[&color_type])?;
                self.stack.push(color_id);
                Ok(())
            },

            PushRgb(r, g, b)                                                => {
                let color_id        = self.stack.last().unwrap();
                let mut insert_rgb  = Self::prepare(&self.sqlite, FloStatement::InsertRgb)?;
                insert_rgb.insert(&[color_id, &(r as f64), &(g as f64), &(b as f64)])?;
                Ok(())
            },

            PushHsluv(h, s, l)                                              => {
                let color_id            = self.stack.last().unwrap();
                let mut insert_hsluv    = Self::prepare(&self.sqlite, FloStatement::InsertHsluv)?;
                insert_hsluv.insert(&[color_id, &(h as f64), &(s as f64), &(l as f64)])?;
                Ok(())
            },

            PopDeleteLayer                                                  => {
                let layer_id            = self.stack.pop().unwrap();
                let mut delete_layer    = Self::prepare(&self.sqlite, FloStatement::DeleteLayer)?;
                delete_layer.execute(&[&layer_id])?;
                Ok(())
            },

            PushLayerType(layer_type)                                       => {
                let layer_type              = self.enum_value(DbEnum::Layer(layer_type));
                let mut insert_layer_type   = Self::prepare(&self.sqlite, FloStatement::InsertLayerType)?;
                let layer_id                = insert_layer_type.insert(&[&layer_type])?;
                self.stack.push(layer_id);
                Ok(())
            },

            PushAssignLayer(assigned_id)                                    => {
                let layer_id                = self.stack.last().unwrap();
                let mut insert_assign_layer = Self::prepare(&self.sqlite, FloStatement::InsertAssignLayer)?;
                insert_assign_layer.insert(&[&self.animation_id, layer_id, &(assigned_id as i64)])?;
                Ok(())
            },

            PushLayerId(layer_id)                                           => {
                self.stack.push(layer_id);
                Ok(())
            },

            PushLayerForAssignedId(assigned_id)                             => {
                let mut select_layer_id = Self::prepare(&self.sqlite, FloStatement::SelectLayerId)?;
                let layer_id            = select_layer_id.query_row(&[&self.animation_id, &(assigned_id as i64)], |row| row.get(0))?;
                self.stack.push(layer_id);
                Ok(())
            },

            PopAddKeyFrame(when)                                            => {
                let layer_id                = self.stack.pop().unwrap();
                let mut insert_key_frame    = Self::prepare(&self.sqlite, FloStatement::InsertKeyFrame)?;
                insert_key_frame.insert(&[&layer_id, &Self::get_micros(&when)])?;
                Ok(())
            },

            PopRemoveKeyFrame(when)                                         => {
                let layer_id                = self.stack.pop().unwrap();
                let mut delete_key_frame    = Self::prepare(&self.sqlite, FloStatement::DeleteKeyFrame)?;
                delete_key_frame.execute(&[&layer_id, &Self::get_micros(&when)])?;
                Ok(())
            },

            PushNearestKeyFrame(when)                                       => {
                let layer_id                        = self.stack.pop().unwrap();
                let mut select_nearest_keyframe     = Self::prepare(&self.sqlite, FloStatement::SelectNearestKeyFrame)?;
                let (keyframe_id, start_micros)     = select_nearest_keyframe.query_row(&[&layer_id, &(Self::get_micros(&when))], |row| (row.get(0), row.get(1)))?;
                self.stack.push(start_micros);
                self.stack.push(keyframe_id);
                Ok(())
            },

            PushVectorElementType(element_type, when)                       => {
                let keyframe_id                     = self.stack.pop().unwrap();
                let start_micros                    = self.stack.pop().unwrap();
                let element_type                    = self.enum_value(DbEnum::VectorElement(element_type));
                let mut insert_vector_element_type  = Self::prepare(&self.sqlite, FloStatement::InsertVectorElementType)?;
                let when                            = Self::get_micros(&when) - start_micros;
                let element_id                      = insert_vector_element_type.insert(&[&keyframe_id, &element_type, &when])?;
                self.stack.push(start_micros);
                self.stack.push(keyframe_id);
                self.stack.push(element_id);
                Ok(())
            },

            PopVectorBrushElement(drawing_style)                            => {
                let brush_id                            = self.stack.pop().unwrap();
                let element_id                          = self.stack.pop().unwrap();
                let drawing_style                       = self.enum_value(DbEnum::DrawingStyle(drawing_style));
                let mut insert_brush_definition_element = Self::prepare(&self.sqlite, FloStatement::InsertBrushDefinitionElement)?;
                insert_brush_definition_element.insert(&[&element_id, &brush_id, &drawing_style])?;
                Ok(())
            },

            PopVectorBrushPropertiesElement                                 => {
                let brush_props_id                  = self.stack.pop().unwrap();
                let element_id                      = self.stack.pop().unwrap();
                let mut insert_brush_props_element  = Self::prepare(&self.sqlite, FloStatement::InsertBrushPropertiesElement)?;
                insert_brush_props_element.insert(&[&element_id, &brush_props_id])?;
                Ok(())
            },

            PopBrushPoints(points)                                          => {
                let element_id              = self.stack.pop().unwrap();
                let mut insert_brush_point  = Self::prepare(&self.sqlite, FloStatement::InsertBrushPoint)?;

                let num_points = points.len();
                for (point, index) in points.iter().zip((0..num_points).into_iter()) {
                    insert_brush_point.insert(&[
                        &element_id, &(index as i64),
                        &(point.cp1.0 as f64), &(point.cp1.1 as f64),
                        &(point.cp2.0 as f64), &(point.cp2.1 as f64),
                        &(point.position.0 as f64), &(point.position.1 as f64),
                        &(point.width as f64)
                    ])?;
                }

                Ok(())
            }
        }
    }

    ///
    /// Performs a set of updates on the database immediately
    /// 
    fn execute_updates_now<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<()> {
        for update in updates {
            self.execute_update(update)?;
        }
        Ok(())
    }
}

impl FloStore for FloSqlite {
    ///
    /// Performs a set of updates on the database
    /// 
    fn update<I: IntoIterator<Item=DatabaseUpdate>>(&mut self, updates: I) -> Result<()> {
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
    fn execute_queue(&mut self) -> Result<()> {
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
    fn flush_pending(&mut self) -> Result<()> {
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

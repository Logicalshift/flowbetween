use super::*;

use std::time::Duration;

///
/// Represents a vector layer in a SQLite database
/// 
pub struct SqliteVectorLayer {
    /// The ID that was assigned to this layer
    assigned_id: u64,

    /// The ID of this layer
    layer_id: i64,

    /// The type of this layer
    layer_type: i64,

    /// Database core
    core: Arc<Desync<AnimationDbCore>>
}

impl AnimationDb {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn get_layer_with_id(&self, assigned_id: u64) -> Option<SqliteVectorLayer> {
        SqliteVectorLayer::from_assigned_id(&self.core, assigned_id)
    }
}

impl SqliteVectorLayer {
    ///
    /// Retrieves a layer for a particular ID
    ///
    pub fn from_assigned_id(core: &Arc<Desync<AnimationDbCore>>, assigned_id: u64) -> Option<SqliteVectorLayer> {
        // Query for the 'real' layer ID
        let layer: Result<(i64, i64)> = core.sync(|core| {
            // Fetch the layer data (we need the 'real' ID here)
            let mut get_layer = core.sqlite.prepare(
                "SELECT Layer.LayerId, Layer.LayerType FROM Flo_AnimationLayers AS Anim \
                        INNER JOIN Flo_LayerType AS Layer ON Layer.LayerId = Anim.LayerId \
                        WHERE Anim.AnimationId = ? AND Anim.AssignedLayerId = ?;")?;
            
            let layer = get_layer.query_row(
                &[&core.animation_id, &(assigned_id as i64)],
                |layer| {
                    (layer.get(0), layer.get(1))
                }
            )?;

            Ok(layer)
        });

        // If the layer exists, create a SqliteVectorLayer
        layer.ok()
            .map(|(layer_id, layer_type)| {
                SqliteVectorLayer {
                    assigned_id:    assigned_id,
                    layer_id:       layer_id,
                    layer_type:     layer_type,
                    core:           Arc::clone(core)
                }
            })
    }
}

impl SqliteVectorLayer {
    ///
    /// Performs an async operation on the database
    /// 
    fn async<TFn: 'static+Send+Fn(&mut AnimationDbCore) -> Result<()>>(&self, action: TFn) {
        self.core.async(move |core| {
            // Only run the function if there has been no failure
            if core.failure.is_none() {
                // Run the function and update the error status
                let result      = action(core);
                core.failure    = result.err();
            }
        })
    }
}

impl Layer for SqliteVectorLayer {
    fn id(&self) -> u64 {
        self.assigned_id
    }

    fn supported_edit_types(&self) -> Vec<LayerEditType> {
        vec![LayerEditType::Vector]
    }

    fn get_frame_at_time(&self, time_index: Duration) -> Arc<Frame> {
        unimplemented!()
    }

    fn get_key_frames(&self) -> Box<Iterator<Item=Duration>> {
        let keyframes = self.core.sync(|core| {
            // Query for the microsecond times from the database
            let mut get_key_frames  = core.sqlite.prepare("SELECT AtTime FROM Flo_LayerKeyFrame WHERE LayerId = ?")?;
            let key_frames          = get_key_frames.query_map(
                &[&self.layer_id],
                |time| { let i: i64 = time.get(0); i }
            )?;

            // Convert to micros to produce the final result
            let key_frames: Vec<Duration> = key_frames
                .map(|micros| AnimationDbCore::from_micros(micros.unwrap()))
                .collect();
            
            Ok(key_frames)
        });

        // Turn into an iterator
        let keyframes = keyframes.unwrap_or_else(|_: Error| vec![]);
        let keyframes = Box::new(keyframes.into_iter());

        keyframes
    }

    fn add_key_frame(&mut self, when: Duration) {
        let layer_id = self.layer_id;

        self.async(move |core| {
            let mut insert_key_frame    = core.sqlite.prepare("INSERT INTO Flo_LayerKeyFrame (LayerId, AtTime) VALUES (?, ?)")?;
            let at_time                 = AnimationDbCore::get_micros(&when);

            insert_key_frame.execute(&[&layer_id, &at_time])?;

            Ok(())
        });
    }

    fn remove_key_frame(&mut self, when: Duration) {
        let layer_id = self.layer_id;

        self.async(move |core| {
            let mut insert_key_frame    = core.sqlite.prepare("DELETE FROM Flo_LayerKeyFrame WHERE LayerId = ? AND AtTime = ?")?;
            let at_time                 = AnimationDbCore::get_micros(&when);

            insert_key_frame.execute(&[&layer_id, &at_time])?;

            Ok(())
        });
    }

    fn as_vector_layer<'a>(&'a self) -> Option<Reader<'a, VectorLayer>> {
        unimplemented!()
    }

    fn edit_vectors<'a>(&'a mut self) -> Option<Editor<'a, VectorLayer>> {
        unimplemented!()
    }
}
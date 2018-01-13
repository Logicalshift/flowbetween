use super::*;

impl AnimationDb {
    ///
    /// Queries the size of the animation that this will edit
    /// 
    pub fn size(&self) -> (f64, f64) {
        // Ask the core for the animation size
        let size = self.core.sync(|core| {
            core.sqlite.query_row("SELECT SizeX, SizeY FROM Flo_Animation WHERE AnimationId = ?", 
                &[&core.animation_id],
                |row| (row.get(0), row.get(1)))
        });

        // TODO: error handling?
        match size {
            Ok((x, y))  => (x, y),
            Err(_)      => (1980.0, 1080.0)
        }
    }

    ///
    /// Queries the active layer IDs for the animation
    /// 
    pub fn get_layer_ids(&self) -> Vec<u64> {
        // Ask the core for the valid layer IDs
        let layer_ids: Result<Vec<i64>> = self.core.sync(|core| {
            let mut get_layers  = core.sqlite.prepare("SELECT LayerId FROM Flo_AnimationLayers WHERE AnimationId = ?")?;
            let layers          = get_layers.query_map(&[&core.animation_id], |row| row.get(0))?;

            let res: Vec<i64>   = layers.map(|layer| layer.unwrap()).collect();

            Ok(res)
        });

        // Convert them
        match layer_ids {
            Ok(ids) => ids.into_iter().map(|id| id as u64).collect(),
            Err(_)  => vec![]
        }
    }
}
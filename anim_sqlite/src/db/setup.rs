use super::*;

const PACKAGE_NAME: &str        = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str     = env!("CARGO_PKG_VERSION");
const V1_DEFINITION: &[u8]      = include_bytes!["../../sql/flo_v1.sqlite"];

impl AnimationDb {
    ///
    /// Initialises the database
    /// 
    pub fn setup(&self) {
        self.async(|core| core.setup());
    }

    ///
    /// Initialises the database
    /// 
    pub fn prepare(&self) {
        self.async(|core| core.prepare());
    }
}

impl AnimationDbCore {
    ///
    /// Initialises the database
    /// 
    pub fn setup(&mut self) -> Result<()> {
        // Create the definition string
        let v1_definition   = String::from_utf8_lossy(V1_DEFINITION);

        // Execute against the database
        self.sqlite.execute_batch(&v1_definition)?;

        // Set the database version string
        let version_string      = format!("{} {}", PACKAGE_NAME, PACKAGE_VERSION);
        let mut update_version  = self.sqlite.prepare("UPDATE FlowBetween SET FloVersion = ?")?;
        update_version.execute(&[&version_string])?;

        Ok(())
    }

    ///
    /// Preprares to use a database that has been setup
    /// 
    pub fn prepare(&mut self) -> Result<()> {
        let animation_id = self.sqlite.query_row("SELECT MIN(AnimationId) FROM Flo_Animation", &[], |row| row.get(0))?;
        self.animation_id = animation_id;

        Ok(())
    }
}

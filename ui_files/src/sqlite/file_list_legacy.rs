use super::file_list::*;
use super::file_error::*;

use flo_logging::*;

use rusqlite::*;
use rusqlite::types::ToSql;

use std::result;

/// Performs the v1 to v2 upgrade steps
const UPGRADE_V1_TO_V2: &[u8]   = include_bytes!["../../sql/file_list_v1_to_v2.sqlite"];

impl FileList {
    ///
    /// Upgrades from version 1 of the database to version 2
    ///
    pub (crate) fn upgrade_v1_to_v2(log: &LogPublisher, connection: &mut Connection) -> result::Result<(), FileListError> {
        log.log((Level::Info, "Upgrading file list from v1 to v2"));

        // Perform the upgrade in a transaction
        let transaction = connection.transaction()?;

        {
            // Create the version table marking this as a v2 database
            transaction.execute_batch(&String::from_utf8_lossy(UPGRADE_V1_TO_V2))?;

            // Assign IDs to everything
            let mut existing_files  = transaction.prepare("SELECT RelativePath FROM Flo_Files")?;
            let existing_files      = existing_files.query_map::<String, _, _>(NO_PARAMS, |file| file.get(0))?;

            let mut file_ids        = vec![];
            let mut add_id          = transaction.prepare("INSERT INTO Flo_Entity_Ordering(NextEntity) VALUES (-1)")?;
            let mut update_id       = transaction.prepare("UPDATE Flo_Files SET EntityId = ? WHERE RelativePath = ?")?;

            for relative_path in existing_files {
                let relative_path = relative_path?;

                // Generate an ID
                let new_id = add_id.insert(NO_PARAMS)?;
                file_ids.push(new_id);

                // Update this file
                update_id.execute::<&[&dyn ToSql]>(&[&new_id, &relative_path])?;
            }

            // Entity ID should now be unique
            transaction.execute_batch("CREATE UNIQUE INDEX Idx_Files_Entity ON Flo_Files (EntityId);")?;

            // Set the file ordering
            let mut set_next_entity = transaction.prepare("UPDATE Flo_Entity_Ordering SET NextEntity = ? WHERE EntityId = ?")?;
            for next_id_idx in 1..file_ids.len() {
                set_next_entity.execute(&[&file_ids[next_id_idx], &file_ids[next_id_idx-1]])?;
            }
        }

        // Commit the transaction
        transaction.commit()?;

        // Upgrade was successful
        Ok(())
    }
}

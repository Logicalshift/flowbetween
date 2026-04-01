use flo_animation::*;
use flo_animation::storage::*;
use flo_sqlite_storage::*;
use flo_ui_files::*;

use futures::prelude::*;

use std::sync::*;
use std::path::{PathBuf};
use std::fmt;
use std::fmt::{Display, Formatter};

///
/// Describes where an animation is stored
///
#[derive(Clone, Debug, PartialEq)]
pub enum StorageDescriptor {
    /// A temporary version of an animation in memory
    InMemory,

    /// A numbered item from the catalog
    CatalogNumber(usize),

    /// A named item from the catalog
    CatalogName(String),

    /// A file with the specified path
    File(String)
}

impl Display for StorageDescriptor {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        use self::StorageDescriptor::*;

        match self {
            InMemory            => write!(fmt, ":inmemory:"),
            CatalogNumber(num)  => write!(fmt, "#{}#", num),
            CatalogName(name)   => write!(fmt, "{}", name),
            File(name)          => write!(fmt, "{}", name)
        }
    }
}

impl StorageDescriptor {
    ///
    /// Opens the animation that this storage descriptor references, using the specified file manager
    ///
    pub fn open_animation(&self, file_manager: &Arc<dyn FileManager>) -> Option<Arc<impl EditableAnimation>> {
        let storage = match self {
            StorageDescriptor::InMemory                 => SqliteAnimationStorage::new_in_memory().ok(),
            StorageDescriptor::File(filename)           => SqliteAnimationStorage::open_file(&PathBuf::from(filename)).ok(),

            StorageDescriptor::CatalogNumber(num)       => {
                let all_files       = file_manager.get_all_files();
                let requested_file  = all_files.into_iter().nth(*num)?;
                SqliteAnimationStorage::open_file(requested_file.as_path()).ok()
            }

            StorageDescriptor::CatalogName(filename)    => {
                let all_files       = file_manager.get_all_files();
                let filename        = filename.to_lowercase();
                let mut result      = None;

                for file in all_files {
                    let full_name = file_manager.display_name_for_path(file.as_path()).unwrap_or("<untitled>".to_string());
                    if full_name.to_lowercase() == filename {
                        result = SqliteAnimationStorage::open_file(file.as_path()).ok();
                        break;
                    }
                }

                result
            }
        };

        let animation   = storage.map(|storage| Arc::new(create_animation_editor(move |commands| storage.get_responses(commands).boxed())));
        animation
    }

    ///
    /// Parses a string that's intended to be a reference to the catalog into a storage descriptor
    ///
    pub fn parse_catalog_string(val: &str) -> StorageDescriptor {
        // The value 'inmemory' just means to use an in-memory file
        if val.to_lowercase() == ":inmemory:" { 
            return StorageDescriptor::InMemory;
        }

        // #n# indicates a catalog number from a file manager
        if val.len() > 2 && val.chars().nth(0) == Some('#') && val.chars().last() == Some('#') {
            // Try to parse as a catalog number
            let number = val.chars().skip(1).take(val.len()-2).collect::<String>();
            if let Ok(number) = number.parse::<usize>() {
                return StorageDescriptor::CatalogNumber(number);
            }
        }

        // Other values are catalog names
        return StorageDescriptor::CatalogName(val.to_string());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_inmemory() {
        assert!(StorageDescriptor::parse_catalog_string(":inmemory:") == StorageDescriptor::InMemory);
    }

    #[test]
    fn parse_number() {
        assert!(StorageDescriptor::parse_catalog_string("#42#") == StorageDescriptor::CatalogNumber(42));
    }

    #[test]
    fn parse_name() {
        assert!(StorageDescriptor::parse_catalog_string("Some name") == StorageDescriptor::CatalogName("Some name".to_string()));
    }
}

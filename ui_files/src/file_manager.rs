///
/// The file manager is used to retrieve what files are available and organize them
/// 
pub trait FileManager : Send+Sync {
    ///
    /// Returns a list of all the files that can be opened by this manager
    /// 
    fn get_all_files(&self) -> Vec<Path>;

    ///
    /// Returns the display name for a particular path
    /// 
    fn display_name_for_path(&self, path: Path) -> Option<String>;

    ///
    /// Reserves a path for a new file
    /// 
    fn create_new_path(&self) -> Path;

    ///
    /// Updates or creates the display name associated with a particular path (which must be
    /// returned via get_all_files: setting a non-existent)
    ///
    fn set_display_name_for_path(&self, path: Path, display_name: String);
}

///
/// Describes where an animation is stored
///
#[derive(Clone, Debug)]
pub enum StorageDescriptor {
    /// A numbered item from the catalog
    CatalogNumber(usize),

    /// A named item from the catalog
    CatalogName(String),

    /// A file with the specified path
    File(String)
}

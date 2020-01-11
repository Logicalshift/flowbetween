///
/// Describes where an animation is stored
///
#[derive(Clone, Debug)]
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

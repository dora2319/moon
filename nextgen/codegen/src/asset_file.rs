use moon_common::path::RelativePathBuf;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub struct AssetFile {
    /// Binary content.
    pub content: Vec<u8>,

    /// Absolute path to destination.
    pub dest_path: PathBuf,

    /// Relative path from templates dir. Also acts as the Tera engine name.
    pub name: RelativePathBuf,

    /// Absolute path to source (in templates dir).
    pub source_path: PathBuf,
}

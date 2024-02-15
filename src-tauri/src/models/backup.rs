use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::PathBuf;
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "bindings/BackupKind.ts")]
pub enum Kind {
    File,
    Directory,
}

impl From<&PathBuf> for Kind {
    fn from(path: &PathBuf) -> Self {
        if path.is_file() {
            Self::File
        } else {
            Self::Directory
        }
    }
}

#[derive(TS, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct Location {
    pub entity_name: String,
    pub path: String,
}

#[derive(TS, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct Options {
    pub use_client_directory: bool,
}

#[derive(TS, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct Backup {
    pub kind: Kind,
    pub client_location: Location,
    pub server_location: Location,
    pub latest_run: Option<u64>,
    pub options: Option<Options>,
}

impl Display for Backup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.client_location.path, self.server_location.path
        )
    }
}

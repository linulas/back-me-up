use serde::Deserialize;
use ts_rs::TS;

#[derive(TS, Deserialize, Clone)]
#[ts(export)]
pub struct Location {
    pub entity_name: String,
    pub path: String,
}

#[derive(TS, Deserialize, Clone)]
#[ts(export)]
pub struct Options {
    pub use_client_directory: bool,
}

#[derive(TS, Deserialize, Clone)]
#[ts(export)]
pub struct Backup {
    pub client_location: Location,
    pub server_location: Location,
    pub latest_run: Option<u64>,
    pub options: Option<Options>,
}

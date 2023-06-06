use crate::ssh::connect::Connection;
use serde::Deserialize;
use tokio::sync::Mutex;
use ts_rs::TS;

use super::backup::Backup;

#[derive(TS, Deserialize)]
#[ts(export)]
pub struct Config {
    pub username: String,
    pub server_address: String,
    pub server_port: u16,
    pub backups: Vec<Backup>,
}

pub struct MutexState {
    pub config: Mutex<Option<Config>>,
    pub connection: Mutex<Option<Connection>>,
}

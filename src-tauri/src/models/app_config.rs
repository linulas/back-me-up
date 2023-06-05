use ts_rs::TS;

use super::backup::Backup;

#[derive(TS)]
#[ts(export)]
pub struct AppConfig {
    pub username: String,
    pub server_address: String,
    pub server_port: u16,
    pub backups: Vec<Backup>,
}

use crate::ssh::connect::{to_home_server, Connection};
use openssh::Session;
use ts_rs::TS;

use super::backup::Backup;

#[derive(TS)]
#[ts(export)]
pub struct Config {
    pub username: String,
    pub server_address: String,
    pub server_port: u16,
    pub backups: Vec<Backup>,
}

pub struct State {
    pub config: Option<Config>,
    pub session: Option<Session>,
    pub connection: Option<Connection>,
}

use crate::jobs::{self, Pool};
use crate::ssh::connect::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError, MutexGuard};
use ts_rs::TS;

#[derive(Debug, Serialize)]
pub enum Error {
    MissingConnection(String),
    Config(String),
    JobPool(String),
}

impl From<PoisonError<MutexGuard<'_, Option<Config>>>> for Error {
    fn from(e: PoisonError<MutexGuard<Option<Config>>>) -> Self {
        Self::Config(e.to_string())
    }
}

impl From<PoisonError<MutexGuard<'_, Pool>>> for Error {
    fn from(e: PoisonError<MutexGuard<Pool>>) -> Self {
        Self::Config(e.to_string())
    }
}

impl From<PoisonError<MutexGuard<'_, HashMap<String, usize>>>> for Error {
    fn from(e: PoisonError<MutexGuard<HashMap<String, usize>>>) -> Self {
        Self::JobPool(e.to_string())
    }
}

#[derive(TS, Deserialize, Clone)]
#[ts(export)]
pub struct Config {
    pub client_name: String,
    pub username: String,
    pub server_address: String,
    pub server_port: u16,
    pub allow_background_backup: bool,
}

pub struct MutexState {
    pub config: Mutex<Option<Config>>,
    pub connection: tokio::sync::Mutex<Option<Connection>>,
    pub jobs: Arc<Mutex<jobs::Active>>,
    pub pool: Mutex<jobs::Pool>,
}

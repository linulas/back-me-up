use crate::jobs::{self, Pool};
use crate::ssh::connect::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use ts_rs::TS;

#[derive(Debug, Serialize)]
pub enum Error {
    MissingConnection(String),
    Config(String),
    JobPool(String),
    Storage(String)
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

impl From<PoisonError<MutexGuard<'_, PathBuf>>> for Error {
    fn from(e: PoisonError<MutexGuard<PathBuf>>) -> Self {
        Self::Storage(e.to_string())
    }
}

#[derive(TS, Serialize, Deserialize, Clone, Debug)]
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
    pub failed_jobs: Arc<Mutex<jobs::Failed>>,
    pub pool: Mutex<jobs::Pool>,
    pub app_cache_dir: Arc<Mutex<PathBuf>>,
    pub app_log_dir: Arc<Mutex<PathBuf>>,
}

use crate::jobs::{self, Pool};
use back_me_up::commands;
use back_me_up::models::app::{self, Config};
use back_me_up::models::backup::Backup;
use back_me_up::models::storage::Folder;
use back_me_up::ssh::{self, connect::Connection};
use log::{debug, info};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, MutexGuard, PoisonError};
use tauri::State;

#[derive(Debug, Serialize)]
pub enum Error {
    App(app::Error),
    Ssh(ssh::Error),
    Job(jobs::Error),
    Command(String),
}

impl From<ssh::Error> for Error {
    fn from(e: ssh::Error) -> Self {
        Self::Ssh(e)
    }
}

impl From<app::Error> for Error {
    fn from(e: app::Error) -> Self {
        Self::App(e)
    }
}

impl From<openssh_sftp_client::Error> for Error {
    fn from(e: openssh_sftp_client::Error) -> Self {
        Self::Ssh(ssh::Error::Sftp(e))
    }
}

impl From<PoisonError<MutexGuard<'_, Option<Config>>>> for Error {
    fn from(e: PoisonError<MutexGuard<Option<Config>>>) -> Self {
        Self::App(app::Error::from(e))
    }
}

impl From<PoisonError<MutexGuard<'_, Pool>>> for Error {
    fn from(e: PoisonError<MutexGuard<Pool>>) -> Self {
        Self::App(app::Error::from(e))
    }
}

impl From<jobs::Error> for Error {
    fn from(e: jobs::Error) -> Self {
        Self::Job(e)
    }
}

impl From<PoisonError<MutexGuard<'_, HashMap<String, usize>>>> for Error {
    fn from(e: PoisonError<MutexGuard<HashMap<String, usize>>>) -> Self {
        Self::App(app::Error::from(e))
    }
}

impl From<PoisonError<MutexGuard<'_, PathBuf>>> for Error {
    fn from(e: PoisonError<MutexGuard<PathBuf>>) -> Self {
        Self::App(app::Error::from(e))
    }
}

impl From<openssh::Error> for Error {
    fn from(e: openssh::Error) -> Self {
        Self::Ssh(ssh::Error::Connection(e))
    }
}

impl From<commands::Error> for Error {
    fn from(e: commands::Error) -> Self {
        Self::Command(e.to_string())
    }
}

#[tauri::command]
pub async fn list_home_folders(state: State<'_, app::MutexState>) -> Result<Vec<Folder>, Error> {
    let connection_mutex_guard = state.connection.lock().await;
    let client = match &connection_mutex_guard.as_ref() {
        Some(connection) => &connection.sftp_client,
        None => {
            let error = app::Error::MissingConnection(String::from("No connection"));
            return Err(Error::App(error));
        }
    };

    let config = match state.config.lock()?.as_ref() {
        Some(config) => config.clone(),
        None => return Err(Error::App(app::Error::Config(String::from("No config")))),
    };

    let username = config.username.clone();
    let client = Arc::clone(&Arc::new(client));
    let result = ssh::commands::list_home_folders(client, username).await?;

    Ok(result)
}

#[tauri::command]
pub async fn set_state(config: Config, state: State<'_, app::MutexState>) -> Result<(), Error> {
    debug!("App cache dir is {:?}", state.app_cache_dir.lock()?);
    debug!("{config:?}");

    if let Some(connection) = state.connection.lock().await.take() {
        info!("Closing connection");
        connection.sftp_client.close().await?;
        connection.ssh_session.close().await?;
    };

    _ = state.config.lock()?.insert(config.clone());
    let app_cache_dir = state.app_cache_dir.lock()?.clone();
    let connection = Connection::new(config, app_cache_dir).await?;
    state.connection.lock().await.get_or_insert(connection);

    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn set_config(config: Config, state: State<'_, app::MutexState>) -> Result<(), Error> {
    _ = state.config.lock()?.insert(config);

    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub async fn backup_entity(
    backup: Backup,
    state: State<'_, app::MutexState>,
) -> Result<String, Error> {
    Ok(jobs::backup::backup_entity(backup, Arc::new(state.inner())).await?)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn drop_pool(state: State<'_, app::MutexState>) -> Result<(), Error> {
    let pool = state.pool.lock()?;
    drop(pool);

    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn backup_on_change(state: State<'_, app::MutexState>, backup: Backup) -> Result<(), Error> {
    Ok(commands::app::backup_on_change(&state, backup)?)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn terminate_background_backup(
    state: State<'_, app::MutexState>,
    backup: Backup,
) -> Result<(), Error> {
    Ok(commands::app::terminate_background_backup(&state, backup)?)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn terminate_all_background_jobs(state: State<'_, app::MutexState>) -> Result<(), Error> {
    let mut jobs = state.jobs.lock()?;
    let mut pool = state.pool.lock()?;
    jobs::backup::terminate_all(&mut jobs, &mut pool);
    Ok(())
}

#[tauri::command]
pub async fn reset(state: State<'_, app::MutexState>) -> Result<(), Error> {
    state.connection.lock().await.take();
    let mut jobs = state.jobs.lock()?;
    let mut pool = state.pool.lock()?;

    jobs::backup::terminate_all(&mut jobs, &mut pool);

    state.config.lock()?.take();
    drop(pool);

    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn start_background_backups(
    state: State<'_, app::MutexState>,
    backups: Vec<Backup>,
) -> Result<(), Error> {
    Ok(commands::app::start_background_backups(&state, backups)?)
}

#[tauri::command]
pub fn get_client_name() -> Result<String, Error> {
    Ok(commands::os::get_hostname()?)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn check_job_status(
    state: State<'_, app::MutexState>,
    id: String,
) -> Result<jobs::Status, Error> {
    Ok(jobs::check_status(id, &state.jobs, &state.failed_jobs)?)
}

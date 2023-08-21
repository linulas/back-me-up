use crate::jobs::{self, Pool};
use bmu::models::app::{self, Config};
use bmu::models::backup::Backup;
use bmu::models::storage::Folder;
use bmu::ssh::{self, connect::Connection};
use glob::{glob, PatternError};
use log::{debug, error, info};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
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

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Command(e.to_string())
    }
}

impl From<PatternError> for Error {
    fn from(e: PatternError) -> Self {
        Self::Command(e.to_string())
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
    let state_config = &state.config.lock()?;
    let config_to_move_into_thread = if let Some(config) = state_config.as_ref() {
        config.clone()
    } else {
        let error = app::Error::Config(String::from("No config detected"));
        return Err(Error::App(error));
    };

    if !config_to_move_into_thread.allow_background_backup {
        return Err(Error::App(app::Error::Config(String::from(
            "Background backup is disabled",
        ))));
    }

    let job_id = jobs::id_from_backup(&backup, &jobs::Kind::BackupOnChange);
    let jobs = Arc::clone(&state.jobs);

    if jobs.lock()?.iter().any(|(id, _)| id == &job_id) {
        info!(
            "Already running background backup for {}",
            backup.client_location.path
        );
        return Ok(());
    };

    let mut pool = state.pool.lock()?;
    pool.execute(move |worker| {
        jobs.lock()
            .expect("Could not lock jobs")
            .insert(job_id, worker.id);
        jobs::backup::directory_on_change(&worker, &backup, config_to_move_into_thread);
    })?;

    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn terminate_background_backup(
    state: State<'_, app::MutexState>,
    backup: Backup,
) -> Result<(), Error> {
    let mut jobs = state.jobs.lock()?;
    let job_id = &jobs::id_from_backup(&backup, &jobs::Kind::BackupOnChange);
    let worker_id = if let Some(id) = jobs.get(job_id) {
        id
    } else {
        let error = jobs::Error::NotFound("could not find job".to_string());
        return Err(Error::Job(error));
    };

    let mut pool = state.pool.lock()?;
    let result = pool.terminate_job(*worker_id, || {
        let file_path = format!("{}/.bmu_event_trigger", backup.client_location.path);

        match Command::new("touch").args([&file_path]).output() {
            Ok(output) => {
                if !output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    let why = format!("Failed to create file: {stdout}\n{stderr}");
                    error!("{why}");
                }
            }
            Err(e) => {
                error!("Failed to create file: {e:?}");
            }
        }

        match Command::new("rm").args([&file_path]).output() {
            Ok(output) => {
                if !output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    let why = format!("Failed to remove file: {stdout}\n{stderr}");
                    error!("{why}");
                }
            }
            Err(e) => {
                error!("Failed to remove file: {e:?}");
            }
        }
    });

    if let Err(e) = result {
        let error = jobs::Error::Terminate(e);
        return Err(Error::Job(error));
    }

    if jobs.remove(job_id).is_some() {
        Ok(())
    } else {
        let error = jobs::Error::Terminate(String::from(
            "Worker was terminated, but couldn't remove job from active list",
        ));
        Err(Error::Job(error))
    }
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
    let state_config = &state.config.lock()?;

    if let Some(config) = state_config.as_ref() {
        if !config.allow_background_backup {
            return Err(Error::App(app::Error::Config(String::from(
                "Background backup is disabled",
            ))));
        }
    }

    let jobs = state.jobs.lock()?;
    let available_workers = state.pool.lock()?.available_workers();
    let backup_jobs_that_are_not_already_running: Vec<_> = backups
        .iter()
        .filter(|b| {
            !jobs
                .iter()
                .any(|(job_id, _)| job_id == &jobs::id_from_backup(b, &jobs::Kind::BackupOnChange))
        })
        .collect();

    let num_of_backups_to_start = backup_jobs_that_are_not_already_running.len();

    if num_of_backups_to_start > available_workers {
        state
            .pool
            .lock()?
            .create_workers(num_of_backups_to_start - available_workers);
    }

    if num_of_backups_to_start == 0 {
        return Ok(());
    }

    state.pool.lock()?.start_all_stopped_workers();

    for value in backup_jobs_that_are_not_already_running {
        let config_to_move_into_thread = if let Some(config) = state_config.as_ref() {
            config.clone()
        } else {
            let error = app::Error::Config(String::from("No config detected"));
            return Err(Error::App(error));
        };

        let backup = value.clone();

        let job_id = jobs::id_from_backup(&backup, &jobs::Kind::BackupOnChange);
        let jobs = Arc::clone(&state.jobs);

        let mut pool = state.pool.lock()?;
        pool.execute(move |worker| {
            jobs.lock()
                .expect("Could not lock jobs")
                .insert(job_id, worker.id);
            jobs::backup::directory_on_change(&worker, &backup, config_to_move_into_thread);
        })?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_client_name() -> Result<String, Error> {
    // TODO: use 'hostname' command for windows
    let uname = Command::new("uname")
        .arg("-n") // -n flag to get the network node hostname
        .output()?;

    if uname.status.success() {
        Ok(String::from_utf8_lossy(&uname.stdout).trim().to_string())
    } else {
        let stdout = String::from_utf8_lossy(&uname.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&uname.stderr).trim().to_string();
        let why = format!("Getting hostname with command 'uname' failed: {stdout}\n{stderr}");
        Err(Error::Command(why))
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn check_job_status(
    state: State<'_, app::MutexState>,
    id: String,
) -> Result<jobs::Status, Error> {
    if state.failed_jobs.lock()?.contains_key(&id) {
        error!("{id}: failed");
        return Ok(jobs::Status::Failed);
    } else if state.jobs.lock()?.contains_key(&id) {
        return Ok(jobs::Status::Running);
    }

    info!("{id}: completed");
    Ok(jobs::Status::Completed)
}

pub fn cleanup_entities_by_pattern(pattern: &str) -> Result<(), Error> {
    for path in (glob(pattern)?).flatten() {
        if path.is_file() {
            fs::remove_file(path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
        }
    }

    info!("cleanup successfull!");
    Ok(())
}

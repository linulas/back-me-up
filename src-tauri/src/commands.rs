use crate::jobs::{self, Pool};
use crate::models::app::{self, Config};
use crate::models::backup::Backup;
use crate::models::folder::{Folder, Size};
use crate::ssh::{self, connect::Connection};
use futures::TryStreamExt;
use openssh_sftp_client::fs::DirEntry;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, MutexGuard, PoisonError};
use tauri::State;

#[derive(Debug, Serialize)]
pub enum Error {
    App(app::Error),
    Ssh(ssh::Error),
    Job(jobs::Error),
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

    let home_dir = client.fs().open_dir(Path::new("./")).await?;
    let entries: Vec<DirEntry> = home_dir.read_dir().try_collect().await?;
    let mut folders: Vec<Folder> = Vec::new();

    for entry in entries {
        let is_dir = match entry.file_type() {
            Some(file_type) => file_type.is_dir(),
            None => {
                continue;
            }
        };

        if !is_dir {
            continue;
        }

        let name = match entry.filename().to_str() {
            Some(name) => name.to_string(),
            None => continue,
        };

        if name.starts_with('.') {
            continue;
        }

        let mutex_guard = match state.config.lock() {
            Ok(guard) => guard,
            Err(e) => {
                println!("Error getting config: {e}");
                continue;
            }
        };

        let config = if let Some(config) = mutex_guard.as_ref() {
            config
        } else {
            println!("config was empty");
            continue;
        };

        let user = &config.username;

        let path = format!("/home/{user}/{name}");

        folders.push(Folder {
            name,
            path,
            size: Some(Size::B(entry.metadata().len().unwrap_or_default())),
        });
    }

    Ok(folders)
}

#[tauri::command]
pub async fn set_state(config: Config, state: State<'_, app::MutexState>) -> Result<(), Error> {
    state.config.lock()?.get_or_insert(config.clone());
    let connection = Connection::new(config).await?;
    state.connection.lock().await.get_or_insert(connection);

    Ok(())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn backup_directory(backup: Backup, state: State<'_, app::MutexState>) -> Result<(), Error> {
    let config_mutex_guard = state.config.lock()?;
    // let x = config_mutex_guard.get_or_insert_default();
    let config = if let Some(config) = config_mutex_guard.as_ref() {
        config
    } else {
        let error = app::Error::MissingConnection(String::from("No connection"));
        return Err(Error::App(error));
    };

    Ok(ssh::commands::backup_to_server(&backup, config)?)
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
    let job_id = jobs::id_from_backup(&backup);
    let jobs = Arc::clone(&state.jobs);

    if jobs.lock()?.iter().any(|(id, _)| id == &job_id) {
        println!(
            "Already running background backup for {}",
            backup.client_folder.path
        );
        return Ok(());
    };

    let mut pool = state.pool.lock()?;
    let state_config = &state.config.lock()?;
    let config_to_move_into_thread = if let Some(config) = state_config.as_ref() {
        config.clone()
    } else {
        let error = app::Error::Config(String::from("No config detected"));
        return Err(Error::App(error));
    };

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
    let job_id = &jobs::id_from_backup(&backup);
    let worker_id = if let Some(id) = jobs.get(job_id) {
        id
    } else {
        let error = jobs::Error::NotFound("could not find job".to_string());
        return Err(Error::Job(error));
    };

    let mut pool = state.pool.lock()?;
    let result = pool.terminate_worker(*worker_id, || {
        let file_path = format!("{}/.bmu_event_trigger", backup.client_folder.path);

        if let Err(e) = Command::new("touch").args([&file_path]).status() {
            println!("Error: {e:?}");
        }

        if let Err(e) = Command::new("rm").args([&file_path]).status() {
            println!("Error: {e:?}");
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
pub async fn reset(state: State<'_, app::MutexState>) -> Result<(), Error> {
    state.connection.lock().await.take();
    state.config.lock()?.take();
    let mut pool = state.pool.lock()?;
    let mut jobs = state.jobs.lock()?;
    for (job_id, worker) in jobs.iter() {
        let client_folder_path = job_id.split('_').nth(0).unwrap();
        println!("Terminating worker: {worker:?}\nPath: {client_folder_path}");

        if let Err(why) = pool.terminate_worker(*worker, || {
            let file_path = format!("{}/.bmu_event_trigger", client_folder_path);

            if let Err(e) = Command::new("touch").args([&file_path]).status() {
                println!("Error: {e:?}");
            }

            if let Err(e) = Command::new("rm").args([&file_path]).status() {
                println!("Error: {e:?}");
            }
        }) {
            println!("Error terminating worker: {why}");
            let error = jobs::Error::Terminate(why.to_string());
            return Err(Error::Job(error));
        };
    }

    jobs.clear();
    drop(pool);

    Ok(())
}

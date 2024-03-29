use super::{id_from_backup, Arguments, Error, Kind, Pool, ThreadAction};
use crate::models::app::{self, Config, MutexState};
use crate::models::backup::{Backup, Location};
use crate::ssh;
use chrono::{DateTime, Local};
use log::{error, info};
use notify::{self, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, MutexGuard};

pub struct WatchDirectory {
    backup: Backup,
    config: app::Config,
}

/// Starts a thread watching a directory for changes and backs up files accordingly.
///
/// # Panics
/// Panics if the directory does not exist, or if the watcher for some reason could not start successfully.
pub fn directory_on_change(worker: &Arguments, backup: &Backup, config: Config) {
    let worker_receiver = worker.receiver.lock().expect("Must have a thread receiver");
    let path = Path::new(&backup.client_location.path);
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(sender, notify::Config::default())
        .expect("failed to create watcher");
    let mut last_modified: DateTime<Local> = Path::new(&backup.client_location.path)
        .metadata()
        .expect("failed to get metadata")
        .modified()
        .expect("expected last modification time (SystemTime) from metadata")
        .into();
    let job = WatchDirectory {
        backup: backup.clone(),
        config,
    };

    if let Err(e) = watcher.watch(path.as_ref(), RecursiveMode::Recursive) {
        error!("failed to watch directory: {e:?}");
        panic!("failed to watch directory");
    }

    info!("watching {}", &backup.client_location.path);

    loop {
        let response = worker_receiver.recv();
        let thread_message = match response {
            Ok(message) => message,
            Err(e) => {
                error!("thread message failed: {e:?}");
                continue;
            }
        };

        match thread_message {
            ThreadAction::Terminate => {
                break;
            }
            ThreadAction::Start | ThreadAction::Continue => (),
        }

        let watcher_res = match receiver.recv() {
            Ok(response) => response,
            Err(e) => {
                error!("notify receiver failed: {e:?}");
                continue;
            }
        };

        match watcher_res {
            Ok(event) => {
                if let Err(e) = handle_notify_event(&event, &job, &mut last_modified) {
                    handle_notify_error(&e, &event, &job);
                }
            }
            Err(e) => error!("notify event failed: {e:?}"),
        };

        match worker.sender.lock() {
            Ok(sender) => {
                if let Err(e) = sender.send(ThreadAction::Continue) {
                    error!("Could not send continue message to thread: {e:?}");
                }
            }
            Err(e) => error!("Could not lock worker sender: {e:?}"),
        };
    }
}

fn handle_notify_error(e: &notify::Error, event: &Event, job: &WatchDirectory) {
    let error_is_of_kind_not_found = match &e.kind {
        notify::ErrorKind::Io(io_error) => matches!(io_error.kind(), io::ErrorKind::NotFound),
        _ => false,
    };

    if !error_is_of_kind_not_found {
        error!("Will not handle the following notify error: {e:?}");
        return;
    }

    if let Some(path_buf) = event.paths.get(0) {
        let target_path = path_buf.to_str().unwrap_or("").replace(' ', r"\ ");
        let server_folder_path = job.backup.server_location.path.replace(' ', r"\ ");
        let client_folder_path = job.backup.client_location.path.replace(' ', r"\ ");
        // make sure to not delete root folder for the backup
        if target_path == server_folder_path {
            info!("Ignoring deletion of {target_path}");
            return;
        }

        let relative_path = target_path.replace(&client_folder_path, "");

        let file_name = path_buf
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string()
            .replace(' ', r"\ ");

        let backup_realtive_to_root = Backup {
            client_location: Location {
                entity_name: file_name.clone(),
                path: target_path,
            },
            server_location: Location {
                entity_name: file_name,
                path: format!(
                    "{}/{}/{}{relative_path}",
                    server_folder_path,
                    job.config.client_name,
                    job.backup.client_location.entity_name
                ),
            },
            latest_run: None,
            options: job.backup.options.clone(),
        };

        info!("Deleting {}", backup_realtive_to_root.server_location.path);
        let result = ssh::commands::delete_from_server(&backup_realtive_to_root, &job.config);

        if let Err(e) = result {
            error!("Could not delete_from_server: {e:?}");
        }
    }
}

fn exec_backup_command(
    event: &Event,
    job: &WatchDirectory,
    latest_modified: &mut DateTime<Local>,
) -> Result<(), notify::Error> {
    if event.paths.get(0).is_none() {
        return Err(notify::Error::path_not_found());
    }

    let path = match event.paths.get(0) {
        Some(value) => value,
        None => return Ok(()),
    };

    if path
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .starts_with('.')
    {
        info!("Ignoring hidden entity: {path:?}");
        return Ok(());
    }

    let ignore_extentions = vec!["sb"];

    if ignore_extentions.contains(
        &path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
    ) {
        info!("Ignoring file with blacklisted extention: {path:?}");
    }

    let data = path.metadata()?;
    let entity_modified_date: DateTime<Local> = data.modified()?.into();
    let root_path = Path::new(&job.backup.client_location.path);
    let new_modified_date: DateTime<Local> = root_path.metadata()?.modified()?.into();

    if new_modified_date.timestamp() <= latest_modified.timestamp() {
        info!("Ignoring already up to date entity: {path:?}");
        return Ok(());
    }

    let relative_path = path
        .to_str()
        .unwrap_or_default()
        .replace(&job.backup.client_location.path, "");

    let server_location_path_without_client_directory = format!(
        "{}/{}{relative_path}",
        job.backup.server_location.path, job.config.client_name,
    );

    let server_location_path = if let Some(option) = &job.backup.options {
        if option.use_client_directory {
            format!(
                "{}/{}/{}{relative_path}",
                job.backup.server_location.path,
                job.config.client_name,
                job.backup.client_location.entity_name
            )
        } else {
            server_location_path_without_client_directory
        }
    } else {
        server_location_path_without_client_directory
    };

    let backup_realtive_to_root = Backup {
        client_location: Location {
            path: path.to_str().unwrap_or_default().to_string(),
            entity_name: job.backup.client_location.entity_name.clone(),
        },
        server_location: Location {
            path: server_location_path,
            entity_name: job.backup.server_location.entity_name.clone(),
        },
        latest_run: None,
        options: job.backup.options.clone(),
    };

    let is_directory = if data.is_dir() {
        info!("Backup directory: {path:?}");
        true
    } else {
        info!("Backup file: {path:?}");
        false
    };
    info!(
        "Server location: {}",
        backup_realtive_to_root.server_location.path
    );

    *latest_modified = entity_modified_date;
    if let Err(e) =
        ssh::commands::backup_to_server(&backup_realtive_to_root, &job.config, is_directory)
    {
        error!("Could not backup: {e:?}");
    }

    Ok(())
}

fn handle_notify_event(
    event: &Event,
    job: &WatchDirectory,
    latest_modified: &mut DateTime<Local>,
) -> Result<(), notify::Error> {
    match event.kind {
        notify::EventKind::Create(_) => exec_backup_command(event, job, latest_modified)?,
        notify::EventKind::Modify(notify::event::ModifyKind::Name(_)) => {
            exec_backup_command(event, job, latest_modified)?;
        }
        _ => (),
    }

    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn terminate_all<S: ::std::hash::BuildHasher>(
    jobs: &mut MutexGuard<HashMap<String, usize, S>>,
    pool: &mut MutexGuard<Pool>,
) {
    jobs.iter_mut().for_each(|(job_id, worker)| {
        info!("Terminating job: {job_id}");
        let client_folder_path = job_id.split('_').next().expect("Could not split job_id");

        if let Err(why) = pool.terminate_job(*worker, || {
            let file_path = format!("{client_folder_path}/.bmu_event_trigger");

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
        }) {
            error!("Could not terminate job: {why}");
        };
    });

    jobs.clear();
}

pub async fn entity_to_server(mut backup: Backup, state: Arc<&MutexState>) -> Result<String, Error> {
    let config_mutex = state.config.lock()?.clone();
    let config = match config_mutex {
        Some(config) => config.clone(),
        None => return Err(Error::App(app::Error::Config(String::from("No config")))),
    };
    let folder_to_assert = format!(
        "./{}/{}",
        backup.server_location.entity_name, config.client_name
    );

    let connection = state.connection.lock().await;
    let connection_ref = connection.as_ref();
    let client = match connection_ref {
        Some(connection) => Arc::new(&connection.sftp_client),
        None => {
            return Err(Error::App(app::Error::MissingConnection(String::from(
                "No connection",
            ))));
        }
    };

    let path = Path::new(&folder_to_assert);
    ssh::commands::assert_client_directory_on_server(&client, path).await?;

    let mut pool = state.pool.lock()?;
    let jobs = Arc::clone(&state.jobs);
    let failed_jobs = Arc::clone(&state.failed_jobs);

    // prepend client_name as a root folder on the server for the backup
    backup.server_location.path = format!("{}/{}", backup.server_location.path, config.client_name);
    let job_id_for_client = id_from_backup(&backup, &Kind::Backup);
    if failed_jobs.lock()?.contains_key(&job_id_for_client) {
        failed_jobs.lock()?.remove(&job_id_for_client);
    }

    pool.execute(move |worker| {
        let job_id = id_from_backup(&backup, &Kind::Backup);
        jobs.lock()
            .expect("Could not lock jobs")
            .insert(job_id.clone(), worker.id);

        match ssh::commands::backup_to_server(&backup, &config, true) {
            Ok(_) => {
                jobs.lock().expect("Could not lock jobs").remove(&job_id);
            }
            Err(e) => {
                error!("{e:?}");
                jobs.lock().expect("Could not lock jobs").remove(&job_id);
                failed_jobs
                    .lock()
                    .expect("Could not lock failed jobs")
                    .insert(job_id, worker.id);
            }
        };
    })?;

    Ok(job_id_for_client)
}

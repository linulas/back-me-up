use super::{Arguments, ThreadAction};
use crate::models::app::{self, Config};
use crate::models::backup::Backup;
use crate::models::folder::Folder;
use crate::ssh;
use chrono::{DateTime, Local};
use notify::{self, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::io;
use std::path::Path;

pub struct WatchDirectory {
    backup: Backup,
    config: app::Config,
}

pub fn directory_on_change(worker: &Arguments, backup: &Backup, config: Config) {
    let thread_receiver = worker.receiver.lock().expect("Must have a thread receiver");
    let path = Path::new(&backup.client_folder.path);
    let (sender, receiver) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(sender, notify::Config::default())
        .expect("failed to create watcher");
    let mut last_modified: DateTime<Local> = Path::new(&backup.client_folder.path)
        .metadata()
        .expect("failed to get metadata")
        .modified()
        .expect("expected last modification time (SystemTime) from metadata")
        .into();
    let job = WatchDirectory {
        backup: backup.clone(),
        config,
    };

    watcher
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .expect("failed to watch directory");

    println!("watching {}", &backup.client_folder.path);

    loop {
        let response = thread_receiver.recv();
        let thread_message = match response {
            Ok(message) => message,
            Err(e) => {
                println!("thread receiver error: {e:?}");
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
                println!("notify receiver error: {e:?}");
                continue;
            }
        };

        match watcher_res {
            Ok(event) => {
                if let Err(e) = handle_notify_event(&event, &job, &mut last_modified) {
                    handle_notify_error(&e, &event, &job);
                }
            }
            Err(e) => println!("notify event error: {e:?}"),
        };

        match worker.sender.lock() {
            Ok(sender) => {
                if let Err(e) = sender.send(ThreadAction::Continue) {
                    println!("Error sending continue message: {e:?}");
                }
            }
            Err(e) => println!("Error locking worker sender: {e:?}"),
        };
    }
}

fn handle_notify_error(e: &notify::Error, event: &Event, job: &WatchDirectory) {
    let error_is_of_kind_not_found = match &e.kind {
        notify::ErrorKind::Io(io_error) => matches!(io_error.kind(), io::ErrorKind::NotFound),
        _ => false,
    };

    if !error_is_of_kind_not_found {
        println!("error handling notify error: {e:?}");
        return;
    }

    if let Some(path_buf) = event.paths.get(0) {
        let target_path = path_buf.to_str().unwrap_or("").replace(' ', r"\ ");
        let server_folder_path = job.backup.server_folder.path.replace(' ', r"\ ");
        let client_folder_path = job.backup.client_folder.path.replace(' ', r"\ ");
        // make sure to not delete root folder for the backup
        if target_path == server_folder_path {
            println!("Ignoring deletion of {target_path}");
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
            client_folder: Folder {
                name: file_name.clone(),
                path: target_path,
                size: None,
            },
            server_folder: Folder {
                name: file_name,
                path: format!(
                    "{}/{}{relative_path}",
                    server_folder_path, job.backup.client_folder.name
                ),
                size: None,
            },
            latest_run: None,
        };

        println!("Deleting {}", backup_realtive_to_root.server_folder.path);
        let result = ssh::commands::delete_from_server(&backup_realtive_to_root, &job.config);

        if let Err(e) = result {
            println!("delete error: {e:?}");
        }
    }
}

pub fn exec_backup_command(
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
        println!("Ignoring hidden entity: {path:?}");
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
        println!("Ignoring file with blacklisted extention: {path:?}");
    }

    let data = path.metadata()?;
    let entity_modified_date: DateTime<Local> = data.modified()?.into();
    let root_path = Path::new(&job.backup.client_folder.path);
    let new_modified_date: DateTime<Local> = root_path.metadata()?.modified()?.into();

    if new_modified_date.timestamp() <= latest_modified.timestamp() {
        println!("Ignoring already up to date entity: {path:?}");
        return Ok(());
    }

    let relative_path = path
        .to_str()
        .unwrap_or_default()
        .replace(&job.backup.client_folder.path, "");

    let backup_realtive_to_root = Backup {
        client_folder: Folder {
            path: path.to_str().unwrap_or_default().to_string(),
            name: job.backup.client_folder.name.clone(),
            size: None,
        },
        server_folder: Folder {
            path: format!(
                "{}/{}{relative_path}",
                job.backup.server_folder.path, job.backup.client_folder.name
            ),
            name: job.backup.server_folder.name.clone(),
            size: None,
        },
        latest_run: None,
    };

    // TODO: make type for FILE
    if data.is_dir() {
        println!("Backup directory: {path:?}");
    } else {
        println!("Backup file: {path:?}");
    }

    *latest_modified = entity_modified_date;
    let result = ssh::commands::backup_to_server(&backup_realtive_to_root, &job.config);
    if let Err(e) = result {
        println!("Error: {e:?}");
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
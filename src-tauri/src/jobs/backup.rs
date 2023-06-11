use chrono::{DateTime, Local};
use notify::{self, Event};
use std::io;
use std::path::Path;

use crate::models::app;
use crate::models::backup::Backup;
use crate::models::folder::Folder;
use crate::ssh;

use super::{watch, Job};

pub fn directory_on_change(backup: Backup, config: app::Config) -> notify::Result<()> {
    watch::job_recursive(Job {
        backup,
        callback: &callback_directory_change,
        handle_callback_error: &handle_backup_callback_error,
        config,
    })
}

fn handle_backup_callback_error(e: notify::Error, event: Event, job: &Job) {
    let error_is_of_kind_not_found = match &e.kind {
        notify::ErrorKind::Io(io_error) => match io_error.kind() {
            io::ErrorKind::NotFound => true,
            _ => false,
        },
        _ => false,
    };

    if !error_is_of_kind_not_found {
        println!("callback error: {e:?}");
        return;
    }

    if let Some(path_buf) = event.paths.get(0) {
        let target_path = path_buf.to_str().unwrap_or("");
        // make sure to not delete root folder for the backup
        if target_path == job.backup.server_folder.path {
            println!("Ignoring deletion of {target_path}");
            return;
        }

        let relative_path = target_path.replace(&job.backup.client_folder.path, &format!("{}", ""));

        let file_name = path_buf
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();

        let backup_realtive_to_root = Backup {
            client_folder: Folder {
                name: file_name.clone(),
                path: path_buf.to_str().unwrap_or_default().to_string(),
                size: None,
            },
            server_folder: Folder {
                name: file_name,
                path: format!("{}{relative_path}", job.backup.server_folder.path),
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

fn callback_directory_change(
    event: &Event,
    job: &Job,
    latest_modified: &mut DateTime<Local>,
) -> Result<(), notify::Error> {
    fn handle_backup(
        event: &Event,
        job: &Job,
        latest_modified: &mut DateTime<Local>,
    ) -> Result<(), notify::Error> {
        if event.paths.get(0).is_none() {
            return Ok(()); // TODO: use error instead
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
        let new_modified_date: DateTime<Local> =
            root_path.metadata().unwrap().modified().unwrap().into();

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
                path: format!("{}/{relative_path}", job.backup.server_folder.path),
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

    match event.kind {
        notify::EventKind::Create(_) => handle_backup(event, &job, latest_modified)?,
        notify::EventKind::Modify(modify_event) => match modify_event {
            notify::event::ModifyKind::Name(_) => handle_backup(event, &job, latest_modified)?,
            _ => (),
        },
        _ => (),
    }

    Ok(())
}

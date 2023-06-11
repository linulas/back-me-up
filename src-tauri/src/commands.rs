use crate::jobs;
use crate::models::app::{self, Config};
use crate::models::backup::Backup;
use crate::models::folder::{Folder, Size};
use crate::ssh::{self, connect::Connection};
use futures::stream::TryStreamExt;
use openssh_sftp_client::fs::DirEntry;
use std::path::Path;
use std::thread;
use tauri::State;

#[tauri::command]
pub async fn list_home_folders(state: State<'_, app::MutexState>) -> Result<Vec<Folder>, ssh::Error> {
    let connection_mutex_guard = state.connection.lock().await;
    let client = &connection_mutex_guard
        .as_ref()
        .expect("Must have a sftp connection")
        .sftp_client;

    let home_dir = client.fs().open_dir(Path::new("./")).await?;
    let entries: Vec<DirEntry> = home_dir.read_dir().try_collect().await?;
    let mut folders: Vec<Folder> = Vec::new();

    for entry in entries {
        if !entry.file_type().expect("entry should be a file").is_dir() {
            continue;
        }

        let name = entry.filename().to_str().unwrap_or_default().to_string();

        if name.starts_with('.') {
            continue;
        }

        let config = state.config.lock().await;
        let user = &config.as_ref().expect("Must have a config").username;

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
pub async fn set_state(config: Config, state: State<'_, app::MutexState>) -> Result<(), ssh::Error> {
    state.config.lock().await.get_or_insert(config.clone());
    let connection = Connection::new(config).await?;
    state.connection.lock().await.get_or_insert(connection);

    Ok(())
}

#[tauri::command]
pub async fn backup_directory(
    backup: Backup,
    state: State<'_, app::MutexState>,
) -> Result<(), ssh::Error> {
    let config_mutex_guard = state.config.lock().await;
    let config = config_mutex_guard
        .as_ref()
        .expect("Must have a sftp connection");

    Ok(ssh::commands::backup_to_server(&backup, config)?)
}

#[tauri::command]
pub async fn backup_on_change(
    state: State<'_, app::MutexState>,
    backup: Backup,
) -> Result<(), ssh::Error> {
    let config_mutex_guard = state.config.lock().await;
    let config = config_mutex_guard
        .as_ref()
        .expect("Must have a sftp connection")
        .clone();

    thread::spawn(move || {
        let result = jobs::backup::directory_on_change(backup, config);

        if let Err(e) = result {
            println!("watch error: {e:?}");
        }
    });

    Ok(())
}

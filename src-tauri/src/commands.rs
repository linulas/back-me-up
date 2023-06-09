use crate::models::app::{self, Config};
use crate::models::backup::Backup;
use crate::models::folder::{Folder, Size};
use crate::ssh::connect::{Connection, Error as ConnectionError};
use futures::stream::TryStreamExt;
use openssh_sftp_client::fs::DirEntry;
use serde::Serialize;
use std::path::Path;
use std::process::Command;
use tauri::State;

#[derive(Debug)]
pub enum Error {
    Sftp(openssh_sftp_client::Error),
    Connection(ConnectionError),
    Command(String),
}

impl From<openssh_sftp_client::Error> for Error {
    fn from(e: openssh_sftp_client::Error) -> Self {
        Self::Sftp(e)
    }
}

impl From<ConnectionError> for Error {
    fn from(e: ConnectionError) -> Self {
        Self::Connection(e)
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{self:?}").serialize(serializer)
    }
}

#[tauri::command]
pub async fn list_home_folders(state: State<'_, app::MutexState>) -> Result<Vec<Folder>, Error> {
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
pub async fn set_state(config: Config, state: State<'_, app::MutexState>) -> Result<(), Error> {
    state.config.lock().await.get_or_insert(config.clone());
    let connection = Connection::new(config).await?;
    state.connection.lock().await.get_or_insert(connection);

    Ok(())
}

#[tauri::command]
pub async fn backup_directory(
    backup: Backup,
    state: State<'_, app::MutexState>,
) -> Result<(), Error> {
    let config_mutex_guard = state.config.lock().await;
    let config = config_mutex_guard
        .as_ref()
        .expect("Must have a sftp connection");

    let connection_string = format!(
        "{}@{}:{}",
        config.username,
        config.server_address.replace("http://", ""),
        backup.server_folder.path
    );

    let scp = Command::new("scp")
        .args([
            "-r",
            "-P",
            &config.server_port.to_string(),
            &backup.client_folder.path,
            &connection_string,
        ])
        .status()
        .expect("Failed to execute scp");

    if scp.success() {
        Ok(())
    } else {
        Err(Error::Command(String::from("SCP command failed")))
    }
}

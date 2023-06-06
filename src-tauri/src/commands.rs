use crate::models::app::{self, Config};
use crate::models::folder::{Folder, Size};
use crate::ssh::connect::{to_home_server, Connection, Error as ConnectionError};
use futures::stream::TryStreamExt;
use openssh_sftp_client::fs::DirEntry;
use serde::Serialize;
use std::path::Path;
use tauri::State;

#[derive(Debug)]
pub enum Error {
    SftpError(openssh_sftp_client::Error),
    ConnectionError(ConnectionError),
}

impl From<openssh_sftp_client::Error> for Error {
    fn from(e: openssh_sftp_client::Error) -> Self {
        Self::SftpError(e)
    }
}

impl From<ConnectionError> for Error {
    fn from(e: ConnectionError) -> Self {
        Self::ConnectionError(e)
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
    let client = &connection_mutex_guard.as_ref().expect("Must have a sftp connection").sftp_client;

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

        let path = format!("/home/{user}/{name}" );

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
    state.config.lock().await.get_or_insert(config);
    let connection = Connection::new(to_home_server().await?).await?;
    state.connection.lock().await.get_or_insert(connection);

    Ok(())
}

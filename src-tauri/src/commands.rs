use crate::models::folder::{Folder, Size};
use crate::ssh::connect::{to_home_server, Connection, Error as ConnectionError};
use futures::stream::TryStreamExt;
use openssh_sftp_client::fs::DirEntry;
use serde::Serialize;
use std::path::Path;

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
pub async fn list_home_folders() -> Result<Vec<Folder>, Error> {
    let ssh_user = std::env::var("SSH_USER").expect("SSH_USER must be set");
    let client = Connection::new(to_home_server().await?).await?.client;
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

        let path = format!("/home/{ssh_user}/{name}");

        folders.push(Folder {
            name,
            path,
            size: Some(Size::B(entry.metadata().len().unwrap_or_default())),
        });
    }

    Ok(folders)
}

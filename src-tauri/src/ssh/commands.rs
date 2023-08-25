use super::Error;
use crate::models::app::Config;
use crate::models::backup::Backup;
use crate::models::storage::{Folder, Size};
use futures::TryStreamExt;
use log::info;
use openssh_sftp_client::Sftp;
use openssh_sftp_client::fs::DirEntry;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

pub async fn assert_client_directory_on_server(client: &Sftp, path: &Path) -> Result<(), Error> {
    match client.open(&path).await {
        Ok(_) => Ok(()),
        Err(e) => {
            info!("Client directory does not exist: {e:?}\nCreating directory...");
            client.fs().create_dir(&path).await?;
            Ok(())
        }
    }
}

pub fn backup_to_server(backup: &Backup, config: &Config, is_directory: bool) -> Result<(), Error> {
    #[allow(unused_variables)]
    let connection_string = format!(
        "{}@{}:{}",
        config.username,
        config.server_address.replace("http://", ""),
        backup.server_location.path
    );

    #[cfg(target_os = "macos")]
    #[allow(unused_variables)]
    let connection_string = format!(
        "{}@{}:'{}'",
        config.username,
        config.server_address.replace("http://", ""),
        backup.server_location.path
    );

    let entity_location_on_client = backup.options.as_ref().map_or_else(
        || backup.client_location.path.clone(),
        |options| {
            if options.use_client_directory || !is_directory {
                backup.client_location.path.clone()
            } else {
                format!("{}/", backup.client_location.path.clone())
            }
        },
    );

    let rsync = Command::new("rsync")
        .arg("-a")
        .arg("-e")
        .arg(format!("ssh -p {}", config.server_port))
        .arg("--exclude=.*")
        .arg(&entity_location_on_client)
        .arg(&connection_string)
        .output()?;

    if rsync.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&rsync.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&rsync.stderr).trim().to_string();
        let why = format!("{stdout}\n{stderr}");
        Err(Error::Command(format!("Rsync failed: {why}")))
    }
}

pub fn delete_from_server(backup: &Backup, config: &Config) -> Result<(), Error> {
    let connection_string = format!(
        "{}@{}",
        config.username,
        config.server_address.replace("http://", ""),
    );

    let delete_command_string = format!("rm -rf {}", backup.server_location.path);

    let ssh_delete = Command::new("ssh")
        .args([
            "-p",
            &config.server_port.to_string(),
            &connection_string,
            &delete_command_string,
        ])
        .output()?;

    if ssh_delete.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&ssh_delete.stdout)
            .trim()
            .to_string();
        let stderr = String::from_utf8_lossy(&ssh_delete.stderr)
            .trim()
            .to_string();
        let why = format!("{stdout}\n{stderr}");
        Err(Error::Command(format!("SSH delete command failed: {why}")))
    }
}

pub async fn list_home_folders(client: Arc<&Sftp>, user: String) -> Result<Vec<Folder>, Error> {
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

        let path = format!("/home/{user}/{name}");

        folders.push(Folder {
            name,
            path,
            size: Some(Size::B(entry.metadata().len().unwrap_or_default())),
        });
    }

    Ok(folders)
}

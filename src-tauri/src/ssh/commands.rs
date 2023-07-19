use super::Error;
use crate::models::app::Config;
use crate::models::backup::Backup;
use log::info;
use openssh_sftp_client::Sftp;
use std::path::Path;
use std::process::Command;

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

pub fn backup_to_server(backup: &Backup, config: &Config) -> Result<(), Error> {
    let connection_string = format!(
        "{}@{}:{}",
        config.username,
        config.server_address.replace("http://", ""),
        backup.server_location.path
    );

    #[cfg(target_os = "macos")]
    let connection_string = format!(
        "{}@{}:'{}'",
        config.username,
        config.server_address.replace("http://", ""),
        backup.server_location.path
    );

    let rsync = Command::new("rsync")
        .arg("-a")
        .arg("-e")
        .arg(format!("ssh -p {}", config.server_port))
        .arg("--exclude=.*")
        .arg(&backup.client_location.path)
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

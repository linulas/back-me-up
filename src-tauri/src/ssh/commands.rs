use super::Error;
use crate::models::app::Config;
use crate::models::backup::Backup;
use openssh_sftp_client::Sftp;
use std::path::Path;
use std::process::Command;

pub async fn assert_client_directory_on_server(client: &Sftp, path: &Path) -> Result<(), Error> {
    match client.open(&path).await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {e:?}");
            client.fs().create_dir(&path).await?;
            Ok(())
        }
    }
}

pub fn backup_to_server(backup: &Backup, config: &Config) -> Result<(), Error> {
    let connection_string = format!(
        "{}@{}:'{}'",
        config.username,
        config.server_address.replace("http://", ""),
        backup.server_folder.path
    );

    let rsync = Command::new("rsync")
        .arg("-av")
        .arg("-e")
        .arg(format!("ssh -p {}", config.server_port))
        .arg("--exclude=.*")
        .arg(&backup.client_folder.path)
        .arg(&connection_string)
        .status()
        .expect("Failed to execute rsync command");

    if rsync.success() {
        Ok(())
    } else {
        Err(Error::Command(String::from("rysnc command failed")))
    }
}

pub fn delete_from_server(backup: &Backup, config: &Config) -> Result<(), Error> {
    let connection_string = format!(
        "{}@{}",
        config.username,
        config.server_address.replace("http://", ""),
    );

    let delete_command_string = format!("rm -rf {}", backup.server_folder.path);

    let ssh_delete = Command::new("ssh")
        .args([
            "-p",
            &config.server_port.to_string(),
            &connection_string,
            &delete_command_string,
        ])
        .status()
        .expect("Failed to execute {delete_command_string}");

    if ssh_delete.success() {
        Ok(())
    } else {
        Err(Error::Command(String::from("SSH delete command failed")))
    }
}

use std::process::Command;

use super::Error;
use crate::models::app::Config;
use crate::models::backup::Backup;

pub fn backup_to_server(backup: &Backup, config: &Config) -> Result<(), Error> {
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

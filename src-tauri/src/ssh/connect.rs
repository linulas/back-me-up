use super::Error;
use log::info;
use openssh::{KnownHosts::Strict, Session, SessionBuilder};
use openssh_sftp_client::{Sftp, SftpOptions};
use std::path::PathBuf;

use crate::models::app::Config;

pub struct Connection {
    pub sftp_client: Sftp,
    pub ssh_session: Session,
}

impl Connection {
    pub async fn new(config: Config, control_directory: PathBuf) -> Result<Self, Error> {
        let options = SftpOptions::new();
        let sftp_client = Sftp::from_session(
            to_server(config.clone(), control_directory.clone()).await?,
            options,
        )
        .await?;
        let ssh_connection = to_server(config, control_directory).await?;
        Ok(Self {
            sftp_client,
            ssh_session: ssh_connection,
        })
    }
}

/// Makes a ssh connection to the specified server
///
/// # Panics
/// If stdout from `whoami` command, which is made after a successful connection, does not match `username` provided in config.
pub async fn to_server(config: Config, control_directory: PathBuf) -> Result<Session, Error> {
    let user = config.username;
    let host = config.server_address;
    let port = config.server_port;
    let session = SessionBuilder::default()
        .known_hosts_check(Strict)
        .control_directory(control_directory)
        .connect(&format!("ssh://{user}@{host}:{port}"))
        .await?;
    info!("Connected as user {user}");

    let whoami = session.command("whoami").output().await?;
    if format!("{user}\n") != String::from_utf8_lossy(&whoami.stdout) {
        return Err(Error::Command(
            "The username on the server does not match the username provided in the config"
                .to_string(),
        ));
    }

    Ok(session)
}

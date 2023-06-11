use super::Error;
use openssh::{KnownHosts::Strict, Session};
use openssh_sftp_client::{Sftp, SftpOptions};

use crate::models::app::Config;

pub struct Connection {
    pub sftp_client: Sftp,
    pub ssh_session: Session,
}

impl Connection {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let options = SftpOptions::new();
        let sftp_client = Sftp::from_session(to_server(config.clone()).await?, options).await?;
        let ssh_connection = to_server(config).await?;
        Ok(Self {
            sftp_client,
            ssh_session: ssh_connection,
        })
    }
}

pub async fn to_server(config: Config) -> Result<Session, Error> {
    let user = config.username;
    let host = config.server_address;
    let port = config.server_port;
    let session = Session::connect(&format!("ssh://{user}@{host}:{port}"), Strict).await?;

    let ls = session.command("ls").output().await?;
    eprintln!(
        "Connected as user {user}\n\n{}",
        String::from_utf8(ls.stdout).expect("server output was not valid UTF-8")
    );

    let whoami = session.command("whoami").output().await?;
    assert_eq!(whoami.stdout, format!("{user}\n").into_bytes());

    Ok(session)
}

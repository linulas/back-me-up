use openssh::{KnownHosts::Strict, Session};

#[derive(Debug)]
pub enum Error {
    ConnectionFailed(openssh::Error),
}

impl From<openssh::Error> for Error {
    fn from(e: openssh::Error) -> Self {
        Self::ConnectionFailed(e)
    }
}

pub struct Connection {
    session: Session,
}

impl Connection {
    pub fn new(session: Session) -> Self {
        Self { session }
    }
}

pub async fn to_home_server() -> Result<Session, Error> {
    let user = std::env::var("SSH_USER").expect("SSH_USER must be set");
    let host = std::env::var("SSH_HOST").expect("SSH_HOST must be set");
    let port = std::env::var("SSH_PORT").expect("SSH_PORT must be set");
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

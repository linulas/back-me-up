use serde::Serialize;

pub mod commands;
pub mod connect;

#[derive(Debug)]
pub enum Error {
    Connection(openssh::Error),
    Sftp(openssh_sftp_client::Error),
    Command(String)
}

impl From<openssh::Error> for Error {
    fn from(e: openssh::Error) -> Self {
        Self::Connection(e)
    }
}

impl From<openssh_sftp_client::Error> for Error {
    fn from(e: openssh_sftp_client::Error) -> Self {
        Self::Sftp(e)
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


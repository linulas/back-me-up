use crate::models::app;
use serde::Serialize;

pub mod commands;
pub mod connect;

#[derive(Debug)]
pub enum Error {
    App(app::Error),
    Connection(openssh::Error),
    Sftp(openssh_sftp_client::Error),
    Command(String),
}

impl From<app::Error> for Error {
    fn from(e: app::Error) -> Self {
        Self::App(e)
    }
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

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Command(e.to_string())
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

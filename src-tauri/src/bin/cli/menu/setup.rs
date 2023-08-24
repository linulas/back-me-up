use crate::storage;
use bmu::commands::os::get_hostname;
use bmu::models::app::{Config, MutexState};
use bmu::ssh::{self, connect::Connection};
use inquire::validator::Validation;
use inquire::{CustomType, InquireError, Text};
use std::io;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::PoisonError;

use super::ui;

#[derive(Debug)]
pub enum Error {
    Inquire(InquireError),
    Io(io::Error),
    SSH(ssh::Error),
    Storage(storage::Error),
}

impl From<InquireError> for Error {
    fn from(err: InquireError) -> Self {
        Self::Inquire(err)
    }
}

impl From<storage::Error> for Error {
    fn from(e: storage::Error) -> Self {
        Self::Storage(e)
    }
}

impl From<ssh::Error> for Error {
    fn from(err: ssh::Error) -> Self {
        Self::SSH(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<openssh_sftp_client::Error> for Error {
    fn from(err: openssh_sftp_client::Error) -> Self {
        Self::SSH(ssh::Error::from(err))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, PathBuf>>> for Error {
    fn from(err: PoisonError<std::sync::MutexGuard<PathBuf>>) -> Self {
        Self::SSH(ssh::Error::App(err.into()))
    }
}

impl From<openssh::Error> for Error {
    fn from(err: openssh::Error) -> Self {
        Self::SSH(ssh::Error::from(err))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, Option<bmu::models::app::Config>>>> for Error {
    fn from(err: PoisonError<std::sync::MutexGuard<Option<bmu::models::app::Config>>>) -> Self {
        Self::SSH(ssh::Error::App(err.into()))
    }
}

pub async fn set_state_and_test_connection(
    state: &MutexState,
    config: Config,
) -> Result<Config, Error> {
    if let Some(connection) = state.connection.lock().await.take() {
        connection.sftp_client.close().await?;
        connection.ssh_session.close().await?;
    };

    _ = state.config.lock()?.insert(config.clone());
    let app_cache_dir = state.app_cache_dir.lock()?.clone();
    let connection = ui::loader(
        "Testing connection...",
        Connection::new(config.clone(), app_cache_dir),
    )
    .await?;
    state.connection.lock().await.get_or_insert(connection);

    Ok(config)
}

fn setup_config() -> Result<Config, Error> {
    let client_name = match get_hostname() {
        Ok(hostname) => hostname,
        Err(_) => String::from("unknown_client"), // TODO: append UID to string
    };
    let validator = |input: &str| {
        if input.chars().count() > 50 {
            Ok(Validation::Invalid(
                "You're only allowed 50 characters.".into(),
            ))
        } else {
            Ok(Validation::Valid)
        }
    };

    let username = Text::new("Backup server username:")
        .with_validator(validator)
        .prompt()?;

    let server_addr_validator = |input: &str| match IpAddr::from_str(input) {
        Ok(_) => Ok(Validation::Valid),
        Err(_) => Ok(Validation::Invalid(
            "Please enter a valid IP address.".into(),
        )),
    };

    let server_address = Text::new("Backup server ip-address:")
        .with_validator(server_addr_validator)
        .prompt()?;

    let server_port = CustomType::<u16>::new("Backup server port:")
        .with_error_message("Please type a valid port number")
        .prompt()?;

    Ok(Config {
        username,
        client_name,
        server_address,
        server_port,
        allow_background_backup: false,
    })
}

pub async fn begin(state: &MutexState) -> Result<Config, Error> {
    loop {
        if let Ok(config) = set_state_and_test_connection(state, setup_config()?).await {
            storage::Storage::load()?.write_conig(config.clone());
            return Ok(config);
        } else {
            println!("⛔️ Could not connect with the provided credentials, try again and make sure your credentials are correct.\n");
        }
    }
}

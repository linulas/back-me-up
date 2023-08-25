use super::storage;
use bmu::jobs::Pool;
use bmu::models::app::{Config, MutexState};
use bmu::{commands, jobs, ssh};
use inquire::InquireError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::{io, process};

mod home;
mod settings;
mod setup;
pub mod ui;

pub enum Action {
    Show,
    Disconnect,
    Exit,
}

#[derive(Debug)]
pub enum Error {
    Inquire(InquireError),
    Io(io::Error),
    Path(String),
    SSH(ssh::Error),
    State(String),
    Job(jobs::Error),
    Storage(storage::Error),
    Command(commands::Error),
}

impl From<openssh_sftp_client::Error> for Error {
    fn from(err: openssh_sftp_client::Error) -> Self {
        Self::SSH(ssh::Error::from(err))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, HashMap<std::string::String, usize>>>> for Error {
    fn from(
        err: PoisonError<std::sync::MutexGuard<'_, HashMap<std::string::String, usize>>>,
    ) -> Self {
        Self::State(err.to_string())
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, Pool>>> for Error {
    fn from(err: PoisonError<std::sync::MutexGuard<Pool>>) -> Self {
        Self::State(err.to_string())
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
impl From<storage::Error> for Error {
    fn from(e: storage::Error) -> Self {
        Self::Storage(e)
    }
}

impl From<commands::Error> for Error {
    fn from(e: commands::Error) -> Self {
        Self::Command(e)
    }
}

impl From<jobs::Error> for Error {
    fn from(e: jobs::Error) -> Self {
        Self::Job(e)
    }
}

impl From<PoisonError<MutexGuard<'_, Option<Config>>>> for Error {
    fn from(e: PoisonError<MutexGuard<Option<Config>>>) -> Self {
        Self::State(e.to_string())
    }
}

impl From<ssh::Error> for Error {
    fn from(err: ssh::Error) -> Self {
        Self::SSH(err)
    }
}

impl From<InquireError> for Error {
    fn from(err: InquireError) -> Self {
        Self::Inquire(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

pub fn handle_inquire_error(err: InquireError) {
    match err {
        InquireError::OperationCanceled => (),
        InquireError::OperationInterrupted => process::exit(0),
        _ => println!("\n⛔️ Error selecting option: {err}"),
    };
}

pub async fn show() {
    let storage = match storage::Storage::load() {
        Ok(s) => s,
        Err(why) => {
            println!("{why:?}");
            panic!("⛔️ Could not load storage");
        }
    };

    let pool = jobs::Pool::new(None);
    let state = MutexState {
        config: Mutex::default(),
        connection: tokio::sync::Mutex::default(),
        jobs: Arc::new(Mutex::default()),
        failed_jobs: Arc::new(Mutex::default()),
        pool: Mutex::new(pool),
        app_cache_dir: Arc::new(Mutex::new(storage.cache_dir.clone())),
    };

    let config = if let Some(c) = storage.config() {
        c
    } else {
        match setup::begin(&state).await {
            Ok(c) => c,
            Err(why) => {
                panic!("⛔️ Could not load config: {why:?}");
            }
        }
    };

    let pattern = format!("{}/.ssh-connection*", storage.cache_dir.display());
    bmu::jobs::fs::cleanup_entities_by_pattern(&pattern).expect("could not cleanup_connections");

    if state
        .config
        .lock()
        .expect("failed to lock config")
        .is_none()
        || state.connection.lock().await.is_none()
    {
        ui::loader(
            "Connecting...",
            setup::set_state_and_test_connection(&state, config.clone()),
        )
        .await
        .expect("Failed to connect to server");
    }

    println!("✅ Connection successfull!\n");

    if config.allow_background_backup {
        commands::app::start_background_backups(
            &state,
            storage.backups().expect("could not load backups"),
        )
        .expect("could not start background backups");
    }

    loop {
        if let Err(why) = home::show(&state).await {
            match why {
                Error::Inquire(err) => handle_inquire_error(err),
                _ => println!("\n⛔️ Error: {why:?}\n"),
            }
        }
    }
}

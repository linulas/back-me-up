use crate::storage;
use bmu::models::app::{Config, MutexState};
use bmu::{jobs, ssh};
use inquire::{error::InquireError, Select};
use std::fmt::Display;
use std::io;
use std::process;
use std::sync::{MutexGuard, PoisonError};

mod backup;

type StartMenuItemText = String;

#[derive(Debug)]
pub enum Error {
    Inquire(InquireError),
    Io(io::Error),
    Path(String),
    SSH(ssh::Error),
    State(String),
    Job(jobs::Error),
    Storage(storage::Error),
}

impl From<storage::Error> for Error {
    fn from(e: storage::Error) -> Self {
        Self::Storage(e)
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

enum StartMenuItem {
    AddBackup(StartMenuItemText),
    RunBackup(StartMenuItemText),
    Exit(StartMenuItemText),
}

impl Display for StartMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::AddBackup(value) | Self::RunBackup(value) | Self::Exit(value) => value,
        };
        write!(f, "{text}")
    }
}

async fn handle_menu_option(option: StartMenuItem, state: &MutexState) -> Result<(), Error> {
    match option {
        StartMenuItem::AddBackup(_) => backup::add(state).await?,
        StartMenuItem::RunBackup(_) => backup::select(state).await?,
        StartMenuItem::Exit(_) => process::exit(0),
    };

    Ok(())
}

pub async fn show(state: &MutexState) -> Result<(), Error> {
    let options = vec![
        StartMenuItem::AddBackup(String::from("Add backup")),
        StartMenuItem::RunBackup(String::from("Run backup")),
        StartMenuItem::Exit(String::from("Exit")),
    ];
    let prompt_result: Result<StartMenuItem, InquireError> = Select::new("Select option", options)
        .with_vim_mode(true)
        .prompt();

    match prompt_result {
        Ok(option) => handle_menu_option(option, state).await?,
        Err(err) => println!("\n⛔️ Error selecting option: {err}"),
    };

    Ok(())
}

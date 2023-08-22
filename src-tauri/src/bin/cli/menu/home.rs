use crate::storage;
use bmu::models::app::{Config, MutexState};
use bmu::models::backup::{Backup, Location};
use bmu::models::storage::Folder;
use bmu::ssh::commands::list_home_folders;
use bmu::{jobs, ssh};
use inquire::{error::InquireError, Select};
use std::fmt::Display;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, MutexGuard, PoisonError};
use std::{process, thread};

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

async fn add_backup(state: &MutexState) -> Result<(), Error> {
    let backup = Backup {
        client_location: get_client_location()?,
        server_location: get_server_location(state).await?,
        latest_run: None,
        options: None,
    };

    println!(
        "Client location: {}\nServer location: {}",
        backup.client_location.path, backup.server_location.path
    );

    let storage = storage::Storage::load()?;
    storage.add_backup(backup.clone())?;
    println!("Backup added to storage");

    let result = jobs::backup::backup_entity(backup, Arc::new(state)).await;

    match result {
        Err(why) => Err(Error::from(why)),
        Ok(id) => {
            print!("\r⏳ Backing up: {id}");
            io::stdout().flush().expect("failed to flush stdout");
            thread::sleep(std::time::Duration::from_secs(1));
            while let jobs::Status::Running =
                jobs::check_status(id.clone(), &state.jobs, &state.failed_jobs)?
            {
                thread::sleep(std::time::Duration::from_millis(500));
            }
            println!("\x1B[1A\x1B[2K");
            io::stdout().flush().expect("failed to flush stdout");
            println!("✅ Backup successfull {}\n", " ".repeat(100));
            Ok(())
        }
    }
}

async fn get_server_location(state: &MutexState) -> Result<Location, Error> {
    let connection = state.connection.lock().await;
    let client = match &connection.as_ref() {
        Some(connection) => &connection.sftp_client,
        None => panic!("No connection"),
    };
    let config = &state.config.lock()?;
    let username = if let Some(config) = config.as_ref() {
        config.username.clone()
    } else {
        return Err(Error::State(String::from("No config detected")));
    };
    let home_folders = list_home_folders(Arc::new(client), username).await?;

    let prompt_result: Result<Folder, InquireError> =
        Select::new("Select server folder", home_folders)
            .with_vim_mode(true)
            .prompt();

    match prompt_result {
        Ok(option) => {
            return Ok(Location {
                path: option.path,
                entity_name: option.name,
            });
        }
        Err(err) => println!("\n⛔️ Error selecting option: {err}"),
    };

    Err(Error::Path(String::from("Could not parse path")))
}

fn get_client_location() -> Result<Location, Error> {
    let mut input = String::new();
    let mut path = PathBuf::new();
    let mut error = String::new();

    while !path.is_dir() {
        input.clear();

        if !error.is_empty() {
            print!("\x1B[2A\x1B[2K");
        }

        print!("Enter absolute path to folder: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;

        path = PathBuf::from(input.trim());

        if !path.is_dir() {
            error = format!("Invalid path: {path:?}");
            println!("{error}");
        } else {
            if !error.is_empty() {
                print!("\x1B[1A\x1B[2K");
            }

            let client_path = match path.to_str() {
                None => return Err(Error::Path(String::from("Could not parse path"))),
                Some(path) => path.to_string(),
            };

            let entity_name = client_path
                .split('/')
                .last()
                .unwrap_or_default()
                .to_string();

            return Ok(Location {
                path: client_path,
                entity_name,
            });
        }
    }

    Err(Error::Path(String::from("Could not parse path")))
}

async fn handle_menu_option(option: StartMenuItem, state: &MutexState) -> Result<(), Error> {
    match option {
        StartMenuItem::AddBackup(_) => add_backup(state).await?,
        StartMenuItem::RunBackup(_) => unimplemented!(),
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

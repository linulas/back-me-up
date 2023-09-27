use super::Error;
use crate::menu::Action;
use crate::{daemon, storage};
use back_me_up::models::app::MutexState;
use back_me_up::models::backup::{Backup, Location, Options};
use back_me_up::models::storage::Folder;
use back_me_up::ssh::commands::list_home_folders;
use back_me_up::{commands, jobs};
use inquire::{Confirm, InquireError, Select};
use std::fmt::Display;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::{io, thread};

type BackupMenuItemText = String;

enum BackupMenuItem {
    Run(BackupMenuItemText),
    Delete(BackupMenuItemText),
    Back(BackupMenuItemText),
}

impl Display for BackupMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Run(text) | Self::Delete(text) | Self::Back(text) => write!(f, "{text}"),
        }
    }
}

enum HandleOrGoBack {
    Handle(Backup),
    Back,
}

impl Display for HandleOrGoBack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Handle(backup) => write!(f, "{backup}"),
            Self::Back => write!(f, "<-- Back"),
        }
    }
}

pub async fn handle(state: &MutexState) -> Result<Action, Error> {
    let backup = match select()? {
        HandleOrGoBack::Handle(backup) => backup,
        HandleOrGoBack::Back => return Ok(Action::Exit),
    };

    let options = vec![
        BackupMenuItem::Run(String::from("Run backup job")),
        BackupMenuItem::Delete(String::from("Delete backup information")),
        BackupMenuItem::Back(String::from("<-- Back")),
    ];

    let option: BackupMenuItem = Select::new("Select option", options)
        .with_vim_mode(true)
        .prompt()?;

    match option {
        BackupMenuItem::Run(_) => run(state, backup).await?,
        BackupMenuItem::Delete(_) => delete(state, &backup)?,
        BackupMenuItem::Back(_) => return Ok(Action::Show),
    };

    Ok(Action::Show)
}

async fn run(state: &MutexState, backup: Backup) -> Result<(), Error> {
    let id = jobs::backup::entity_to_server(backup.clone(), Arc::new(state)).await?;
    print!("\r⏳ Backing up: {id}");
    io::stdout().flush().expect("failed to flush stdout");
    thread::sleep(std::time::Duration::from_secs(1));
    while matches!(
        jobs::check_status(&id, &state.jobs, &state.failed_jobs)?,
        jobs::Status::Running
    ) {
        thread::sleep(std::time::Duration::from_millis(500));
    }
    println!("\x1B[1A\x1B[2K");
    io::stdout().flush().expect("failed to flush stdout");

    if matches!(
        jobs::check_status(&id, &state.jobs, &state.failed_jobs)?,
        jobs::Status::Failed
    ) {
        print!("\r{}", " ".repeat(100)); // removes the loading indicator
        return Err(Error::Job(jobs::Error::Failed(format!(
            "Something went wrong when backing up {}",
            backup.client_location.path
        ))));
    }

    println!("✅ Backup successfull {}\n", " ".repeat(100));
    Ok(())
}

fn select() -> Result<HandleOrGoBack, Error> {
    let storage = storage::Storage::load()?;
    let backups = storage.backups()?;
    let mut options: Vec<HandleOrGoBack> = backups
        .iter()
        .map(|backup| HandleOrGoBack::Handle(backup.clone()))
        .collect();

    options.push(HandleOrGoBack::Back);

    let storage = storage::Storage::load()?;

    let option: HandleOrGoBack = if daemon::is_running(&storage) {
        Select::new("Select a backup", options)
            .with_vim_mode(true)
            .with_help_message("Daemon is running, remember to run 'bmu daemon restart' if you make changes in your backups")
            .prompt()?
    } else {
        Select::new("Select a backup", options)
            .with_vim_mode(true)
            .prompt()?
    };

    Ok(option)
}

pub async fn add(state: &MutexState) -> Result<Action, Error> {
    let backup = Backup {
        client_location: get_client_location()?,
        server_location: get_server_location(state).await?,
        latest_run: None,
        options: Some(get_options()?),
    };

    println!(
        "Client location: {}\nServer location: {}",
        backup.client_location.path, backup.server_location.path
    );

    let storage = storage::Storage::load()?;
    storage.add_backup(backup.clone())?;
    println!("Backup added to storage");

    run(state, backup.clone()).await?;

    let config = if let Some(config) = state.config.lock()?.as_ref() {
        config.clone()
    } else {
        return Err(Error::State(String::from("No config detected")));
    };

    if config.allow_background_backup {
        commands::app::backup_on_change(state, backup)?;
    }

    Ok(Action::Show)
}

fn delete(state: &MutexState, backup: &Backup) -> Result<(), Error> {
    let storage = storage::Storage::load()?;
    if let Some(config) = state.config.lock()?.as_ref() {
        if config.allow_background_backup {
            commands::app::terminate_background_backup(state, backup)?;
        }
    }
    Ok(storage.delete_backup(backup)?)
}

/// Prompts the user to select a server location. The prompt will list all root folders in the
/// users home directory.
///
/// # Panics
/// If there is no connection to the server.
async fn get_server_location(state: &MutexState) -> Result<Location, Error> {
    let connection = state.connection.lock().await;
    let connection_ref = connection.as_ref();
    let client = connection_ref.map_or_else(|| panic!("No connection"), |c| &c.sftp_client);
    let config = state.config.lock()?.clone();
    let username = if let Some(config) = config {
        config.username
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

        if path.is_dir() {
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
        error = format!("Invalid path: {path:?}");
    }

    Err(Error::Path(format!("Could not parse path\n{error}")))
}

fn get_options() -> Result<Options, Error> {
    let use_client_directory =
        Confirm::new("Use client directory as top level on the backup server?")
            .with_default(false)
            .prompt()?;

    Ok(Options {
        use_client_directory,
    })
}

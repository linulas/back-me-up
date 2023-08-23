use super::Error;
use crate::storage;
use bmu::jobs;
use bmu::models::app::MutexState;
use bmu::models::backup::{Backup, Location};
use bmu::models::storage::Folder;
use bmu::ssh::commands::list_home_folders;
use inquire::{InquireError, Select};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::{io, thread};

pub async fn run(state: &MutexState, backup: Backup) -> Result<(), Error> {
    let id = jobs::backup::backup_entity(backup.clone(), Arc::new(state)).await?;
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

    if let jobs::Status::Failed = jobs::check_status(id.clone(), &state.jobs, &state.failed_jobs)? {
        print!("\r{}", format!("{}", " ".repeat(100))); // removes the loading indicator
        return Err(Error::Job(jobs::Error::Failed(format!(
            "Something went wrong when backing up {}",
            backup.client_location.path
        ))));
    }

    println!("✅ Backup successfull {}\n", " ".repeat(100));
    Ok(())
}

pub async fn select(state: &MutexState) -> Result<(), Error> {
    let storage = storage::Storage::load()?;
    let backups = storage.backups()?;

    let backup: Backup = Select::new("Select a backup", backups)
        .with_vim_mode(true)
        .prompt()?;

    Ok(run(state, backup).await?)
}

pub async fn add(state: &MutexState) -> Result<(), Error> {
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

    Ok(run(state, backup).await?)
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

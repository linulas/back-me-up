use super::{Action, Error};
use crate::storage;
use bmu::models::app::MutexState;
use bmu::{commands, jobs};
use inquire::{Confirm, Select};
use std::fmt::Display;

enum SettingsMenuItem {
    EnableBackgroundBackups(String),
    DisableBackgroundBackups(String),
    Disconnect(String),
    Back(String),
}

impl Display for SettingsMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::EnableBackgroundBackups(value)
            | Self::DisableBackgroundBackups(value)
            | Self::Disconnect(value)
            | Self::Back(value) => value,
        };
        write!(f, "{text}")
    }
}

fn set_background_backups(enabled: bool, state: &MutexState) -> Result<Action, Error> {
    let storage = storage::Storage::load()?;
    let mut config = match state.config.lock()?.as_ref() {
        Some(config) => config.clone(),
        None => return Err(Error::State(String::from("No config found"))),
    };

    config.allow_background_backup = enabled;
    storage.write_conig(config.clone());
    _ = state.config.lock()?.insert(config);

    if enabled {
        commands::app::start_background_backups(state, storage.backups()?)?;
    } else {
        let mut jobs = state.jobs.lock()?;
        let mut pool = state.pool.lock()?;

        jobs::backup::terminate_all(&mut jobs, &mut pool);
    }

    Ok(Action::Show)
}

async fn disconnect(state: &MutexState) -> Result<Action, Error> {
    let confirmed = Confirm::new("Are you sure you want to erase your credentials and disconnect?\nYour backups will remain on the server").with_default(true).prompt()?;

    if !confirmed {
        return Ok(Action::Show);
    }

    if let Some(connection) = state.connection.lock().await.take() {
        connection.sftp_client.close().await?;
        connection.ssh_session.close().await?;
    }

    let mut jobs = state.jobs.lock()?;
    let mut pool = state.pool.lock()?;
    jobs::backup::terminate_all(&mut jobs, &mut pool);

    state.config.lock()?.take();
    drop(pool);

    let storage = storage::Storage::load()?;
    storage.reset()?;

    println!("\n🔴 Disconnected\n");

    Ok(Action::Disconnect)
}

pub async fn show(state: &MutexState) -> Result<Action, Error> {
    let allow_background_backups = match state.config.lock()?.as_ref() {
        Some(config) => config.allow_background_backup,
        None => false,
    };
    let options = vec![
        if allow_background_backups {
            SettingsMenuItem::DisableBackgroundBackups(String::from("Disable background backups"))
        } else {
            SettingsMenuItem::EnableBackgroundBackups(String::from("Enable background backups"))
        },
        SettingsMenuItem::Disconnect(String::from("Disconnnect")),
        SettingsMenuItem::Back(String::from("Back")),
    ];

    let option: SettingsMenuItem = Select::new("Select option", options)
        .with_vim_mode(true)
        .prompt()?;

    match option {
        SettingsMenuItem::EnableBackgroundBackups(_) => set_background_backups(true, state),
        SettingsMenuItem::DisableBackgroundBackups(_) => set_background_backups(false, state),
        SettingsMenuItem::Disconnect(_) => disconnect(state).await,
        SettingsMenuItem::Back(_) => return Ok(Action::Exit),
    }
}

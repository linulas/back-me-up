use super::{handle_inquire_error, settings, Action, Error};
use bmu::models::app::MutexState;
use inquire::{error::InquireError, Select};
use std::fmt::Display;
use std::process;

mod backup;

type StartMenuItemText = String;

enum StartMenuItem {
    AddBackup(StartMenuItemText),
    HandleBackups(StartMenuItemText),
    Settings(StartMenuItemText),
    Exit(StartMenuItemText),
}

impl Display for StartMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::AddBackup(value)
            | Self::HandleBackups(value)
            | Self::Settings(value)
            | Self::Exit(value) => value,
        };
        write!(f, "{text}")
    }
}

async fn handle_menu_option(option: StartMenuItem, state: &MutexState) -> Result<(), Error> {
    match option {
        StartMenuItem::AddBackup(_) => backup::add(state).await?,
        StartMenuItem::HandleBackups(_) => loop {
            let action = backup::handle(state).await?;
            if let Action::Exit = action {
                break;
            }
        },
        StartMenuItem::Settings(_) => loop {
            let action = settings::show(state).await?;
            if let Action::Exit = action {
                break;
            }
            if let Action::Disconnect = action {
                process::exit(0);
            }
        },
        StartMenuItem::Exit(_) => process::exit(0),
    };

    Ok(())
}

pub async fn show(state: &MutexState) -> Result<(), Error> {
    let options = vec![
        StartMenuItem::AddBackup(String::from("Add backup")),
        StartMenuItem::HandleBackups(String::from("Handle backups")),
        StartMenuItem::Settings(String::from("Settings")),
        StartMenuItem::Exit(String::from("Exit")),
    ];
    let prompt_result: Result<StartMenuItem, InquireError> = Select::new("Select option", options)
        .with_vim_mode(true)
        .prompt();

    match prompt_result {
        Ok(option) => handle_menu_option(option, state).await?,
        Err(err) => handle_inquire_error(err),
    };

    Ok(())
}

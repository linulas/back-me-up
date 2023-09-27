use super::{settings, Action, Error};
use back_me_up::models::app::MutexState;
use inquire::Select;
use std::fmt::Display;

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

pub async fn show(state: &MutexState) -> Result<Action, Error> {
    let options = vec![
        StartMenuItem::AddBackup(String::from("Add backup")),
        StartMenuItem::HandleBackups(String::from("Handle backups")),
        StartMenuItem::Settings(String::from("Settings")),
        StartMenuItem::Exit(String::from("Exit")),
    ];
    let option: StartMenuItem = Select::new("Select option", options)
        .with_vim_mode(true)
        .prompt()?;

    match option {
        StartMenuItem::AddBackup(_) => backup::add(state).await,
        StartMenuItem::HandleBackups(_) => loop {
            let action = backup::handle(state).await?;
            if matches!(action, Action::Exit) {
                return Ok(Action::Show);
            }
        },
        StartMenuItem::Settings(_) => loop {
            let action = settings::show(state).await?;
            if matches!(action, Action::Exit) {
                return Ok(Action::Show);
            } else if matches!(action, Action::Disconnect) {
                return Ok(Action::Exit);
            }
        },
        StartMenuItem::Exit(_) => Ok(Action::Exit),
    }
}

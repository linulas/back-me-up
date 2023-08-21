use inquire::{error::InquireError, Select};
use std::fmt::Display;

type StartMenuItemText = String;

enum StartMenuItem {
    AddBackup(StartMenuItemText),
    RunBackup(StartMenuItemText),
}

impl Display for StartMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::AddBackup(value)
            | Self::RunBackup(value) => value,
        };
        write!(f, "{text}")
    }
}

fn handle_menu_option(option: StartMenuItem) {
    // let title = format!("{option} 🚀");
    match option {
        StartMenuItem::AddBackup(_) => unimplemented!(),
        StartMenuItem::RunBackup(_) => unimplemented!(),
    };
}

pub fn show() {
    let options = vec![
        StartMenuItem::AddBackup(String::from("Add backup")),
        StartMenuItem::RunBackup(String::from("Run backup")),
    ];
    let prompt_result: Result<StartMenuItem, InquireError> = Select::new("Select option", options)
        .with_vim_mode(true)
        .prompt();

    match prompt_result {
        Ok(option) => handle_menu_option(option),
        Err(err) => println!("\n⛔️ Error selecting option: {err}"),
    };
}


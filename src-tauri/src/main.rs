// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio::sync::Mutex;

use self::models::app::MutexState;

mod commands;
mod jobs;
mod models;
mod ssh;

#[cfg(test)]
mod tests;

fn main() {
    tauri::Builder::default()
        .manage(MutexState {
            config: Mutex::default(),
            connection: Mutex::default(),
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_home_folders,
            commands::set_state,
            commands::backup_directory,
            commands::backup_on_change
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

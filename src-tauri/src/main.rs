// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use self::models::app::MutexState;
use std::sync::{Mutex, Arc};

mod commands;
mod jobs;
mod models;
mod ssh;

#[cfg(test)]
mod tests;

fn main() {
    let pool = jobs::Pool::new(10);

    tauri::Builder::default()
        .manage(MutexState {
            config: Mutex::default(),
            connection: tokio::sync::Mutex::default(),
            jobs: Arc::new(Mutex::default()),
            pool: Mutex::new(pool),
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_home_folders,
            commands::set_state,
            commands::backup_directory,
            commands::backup_on_change,
            commands::terminate_background_backup,
            commands::drop_pool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

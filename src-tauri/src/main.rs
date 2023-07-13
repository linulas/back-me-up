// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use self::models::app::MutexState;
use std::sync::{Arc, Mutex};
use tauri::SystemTray;
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem};

mod commands;
mod event;
mod jobs;
mod models;
mod ssh;
mod tray;

#[cfg(test)]
mod tests;

fn main() {
    let open_dashboard = CustomMenuItem::new("open".to_string(), "Open dashboard");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(open_dashboard)
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let app_tray = SystemTray::new().with_menu(tray_menu);
    let pool = jobs::Pool::new(None);

    tauri::Builder::default()
        .manage(MutexState {
            config: Mutex::default(),
            connection: tokio::sync::Mutex::default(),
            jobs: Arc::new(Mutex::default()),
            failed_jobs: Arc::new(Mutex::default()),
            pool: Mutex::new(pool),
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_home_folders,
            commands::set_state,
            commands::set_config,
            commands::backup_directory,
            commands::start_background_backups,
            commands::backup_on_change,
            commands::terminate_background_backup,
            commands::terminate_all_background_jobs,
            commands::drop_pool,
            commands::reset,
            commands::get_client_name,
            commands::check_job_status
        ])
        .system_tray(app_tray)
        .on_system_tray_event(tray::handle_system_tray_event)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(event::handle_tauri_run);
}

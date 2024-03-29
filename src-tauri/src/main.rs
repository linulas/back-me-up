// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use back_me_up::{jobs, models::app::MutexState};
use log::{warn, LevelFilter};
use std::fs::DirBuilder;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::SystemTray;
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem};
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_plugin_log::LogTarget;

mod event;
mod handlers;
mod tray;

#[cfg(debug_assertions)]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::Webview];
#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::Warn;
#[cfg(not(debug_assertions))]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::LogDir];

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
    let init_cache_dir: Arc<Mutex<PathBuf>> = Arc::new(Mutex::default());
    let init_log_dir: Arc<Mutex<PathBuf>> = Arc::new(Mutex::default());
    let app_cache_dir_for_setup = Arc::clone(&init_cache_dir);
    let app_log_dir_for_setup = Arc::clone(&init_cache_dir);

    tauri::Builder::default()
        .setup(move |app| {
            let app_cache_dir = app.path_resolver().app_cache_dir().map_or_else(
                || {
                    warn!("Could not find app cache directory");
                    PathBuf::from("./")
                },
                |dir| {
                    if !dir.exists() {
                        DirBuilder::new()
                            .create(&dir)
                            .expect("could not create app cache directory");
                    }
                    dir
                },
            );
            let app_log_dir = app.path_resolver().app_log_dir().map_or_else(
                || {
                    warn!("Could not find app log directory");
                    PathBuf::from("./")
                },
                |dir| {
                    if !dir.exists() {
                        DirBuilder::new()
                            .create(&dir)
                            .expect("could not create app cache directory");
                    }
                    dir
                },
            );

            *app_cache_dir_for_setup
                .lock()
                .expect("could not lock app cache dir on setup") = app_cache_dir;
            *app_log_dir_for_setup
                .lock()
                .expect("could not lock app cache dir on setup") = app_log_dir;
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets(LOG_TARGETS)
                .with_colors(ColoredLevelConfig::default())
                .level(LOG_LEVEL)
                .build(),
        )
        .manage(MutexState {
            config: Mutex::default(),
            connection: tokio::sync::Mutex::default(),
            jobs: Arc::new(Mutex::default()),
            failed_jobs: Arc::new(Mutex::default()),
            pool: Mutex::new(pool),
            app_cache_dir: Arc::clone(&init_cache_dir),
            app_log_dir: Arc::clone(&init_log_dir),
        })
        .invoke_handler(tauri::generate_handler![
            handlers::list_home_folders,
            handlers::set_state,
            handlers::set_config,
            handlers::backup_entity,
            handlers::start_background_backups,
            handlers::backup_on_change,
            handlers::terminate_background_backup,
            handlers::terminate_all_background_jobs,
            handlers::drop_pool,
            handlers::reset,
            handlers::get_client_name,
            handlers::check_job_status
        ])
        .system_tray(app_tray)
        .on_system_tray_event(tray::handle_system_tray_event)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(event::handle_tauri_run);
}

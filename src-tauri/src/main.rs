// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use self::models::app::MutexState;
use std::sync::{Arc, Mutex};
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem};
use tauri::{Manager, SystemTray, SystemTrayEvent};

mod commands;
mod jobs;
mod models;
mod ssh;

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
    let tray = SystemTray::new().with_menu(tray_menu);
    let pool = jobs::Pool::new(None);

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
            commands::reset,
        ])
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "open" => {
                    if let Some(main_window) = app.get_window("main") {
                        if let Err(e) = main_window.set_focus() {
                            println!("failed to focus main window: {e:?}");
                        }
                    } else {
                        tauri::WindowBuilder::new(
                            &app.app_handle(),
                            "main",
                            tauri::WindowUrl::App("index.html".into()),
                        )
                        .title("BMU")
                        .resizable(true)
                        .fullscreen(false)
                        .inner_size(800.0, 600.0)
                        .build()
                        .expect("failed to build main window");
                    };
                }
                "settings" => {
                    tauri::WindowBuilder::new(
                        &app.app_handle(),
                        "settings",
                        tauri::WindowUrl::App("/settings".into()),
                    )
                    .title("Settings")
                    .resizable(false)
                    .inner_size(600.0, 400.0)
                    .build()
                    .expect("failed to build settings window");
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}

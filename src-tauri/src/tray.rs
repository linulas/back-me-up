use back_me_up::graceful_exit;
use log::error;
use log::warn;
use tauri::AppHandle;
use tauri::Manager;
use tauri::SystemTrayEvent;
use tokio::runtime::Runtime;

fn create_main_window(app: &AppHandle) {
    app.get_window("main").map_or_else(
        || {
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
        },
        |main_window| {
            if let Err(e) = main_window.set_focus() {
                error!("failed to focus main window: {e:?}");
            }
        },
    );
}

fn create_settings_window(app: &AppHandle) {
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

pub fn handle_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    if let SystemTrayEvent::MenuItemClick { id, .. } = event {
        match id.as_str() {
            "open" => create_main_window(app),
            "settings" => create_settings_window(app),
            "quit" => {
                let rt = Runtime::new().unwrap();
                let handle = rt.handle();
                handle.block_on(graceful_exit(&app.state()));
                app.path_resolver().app_cache_dir().map_or_else(
                    || {
                        warn!("Could not find app cache directory");
                    },
                    |app_cache_dir| {
                        let pattern = format!("{}/.ssh-connection*", app_cache_dir.display());
                        back_me_up::jobs::fs::cleanup_entities_by_pattern(&pattern)
                            .expect("could not cleanup_connections");
                    },
                );
            }
            _ => {}
        }
    }
}

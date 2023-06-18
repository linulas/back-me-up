use tauri::AppHandle;
use tauri::Manager;
use tauri::SystemTrayEvent;

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
                println!("failed to focus main window: {e:?}");
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
                std::process::exit(0);
            }
            _ => {}
        }
    }
}

use tauri::{AppHandle, RunEvent};

pub fn handle_tauri_run(_: &AppHandle, event: RunEvent) {
    if let RunEvent::ExitRequested { api, .. } = event {
        api.prevent_exit();
    }
}

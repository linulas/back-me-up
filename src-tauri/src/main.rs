// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;

mod commands;
mod models;
mod ssh;

#[cfg(test)]
mod tests;

fn main() {
    dotenv().ok();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::list_home_folders])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

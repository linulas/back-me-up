mod menu;
mod storage;
use std::env;

static USER_PANIC: &str = "USER not set";
static BUNDLE_IDENTIFIER: &str = "BackMeUp";

#[tokio::main]
async fn main() {
    let os = env::consts::OS;
    let user = env::var_os("USER").expect(USER_PANIC);

    if user.is_empty() {
        panic!("{USER_PANIC}");
    }

    let home_dir_os_string = if let Some(dir) = env::var_os("HOME") {
        dir.clone()
    } else if let Some(dir) = env::var_os("USERPROFILE") {
        dir.clone()
    } else {
        panic!("Unable to determine user's home directory");
    };

    let home_dir = home_dir_os_string.to_str().expect("HOME not set");

    // INFO: Based on tauri path: https://tauri.app/v1/api/js/path/#functions
    if os == "linux" {
        env::set_var(
            "APP_CACHE_DIR",
            format!("{home_dir}/.cache/{BUNDLE_IDENTIFIER}"),
        );
        env::set_var(
            "APP_CONFIG_DIR",
            format!("{home_dir}/.config/{BUNDLE_IDENTIFIER}"),
        );
        env::set_var(
            "APP_DATA_DIR",
            format!("{home_dir}/.local/share/{BUNDLE_IDENTIFIER}"),
        );
    } else if os == "macos" {
        env::set_var(
            "APP_CACHE_DIR",
            format!("{home_dir}/Library/Caches/{BUNDLE_IDENTIFIER}"),
        );
        env::set_var(
            "APP_CONFIG_DIR",
            format!("{home_dir}/Library/Application Support/{BUNDLE_IDENTIFIER}"),
        );
        env::set_var(
            "APP_DATA_DIR",
            format!("{home_dir}/Library/Application Support/{BUNDLE_IDENTIFIER}"),
        );
    } else {
        panic!("Unsupported OS");
    }

    // Print the environment variable
    let cache_dir = env::var("APP_CACHE_DIR").expect("APP_CACHE_DIR not set");
    let config_dir = env::var("APP_CONFIG_DIR").expect("APP_CONFIG_DIR not set");
    let data_dir = env::var("APP_DATA_DIR").expect("APP_DATA_DIR not set");

    let messages = vec![
        format!("USER: {}", user.to_str().expect(USER_PANIC)),
        format!("APP_CACHE_DIR: {cache_dir}"),
        format!("APP_CONFIG_DIR: {config_dir}"),
        format!("APP_DATA_DIR: {data_dir}"),
    ];

    menu::ui::print_frame("Back me up 🚀", messages);

    menu::show().await;
}

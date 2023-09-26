use self::menu::ui;
use back_me_up::commands::os::{create_directory, directory_exists};
use back_me_up::jobs::Pool;
use back_me_up::models::app::Config as AppConfig;
use back_me_up::models::app::MutexState;
use back_me_up::ssh::connect::Connection;
use back_me_up::{commands, jobs, ssh};
use inquire::InquireError;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{MutexGuard, PoisonError};
use std::{env, io};

mod daemon;
mod menu;
mod storage;

static USER_PANIC: &str = "USER not set";
static BUNDLE_IDENTIFIER: &str = "BackMeUp";
static TITLE: &str = "Back me up 🚀";

#[derive(Debug)]
pub enum Error {
    Inquire(InquireError),
    Io(io::Error),
    Path(String),
    SSH(ssh::Error),
    State(String),
    Job(jobs::Error),
    Storage(storage::Error),
    Command(commands::Error),
}

impl From<openssh_sftp_client::Error> for Error {
    fn from(err: openssh_sftp_client::Error) -> Self {
        Self::SSH(ssh::Error::from(err))
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, HashMap<std::string::String, usize>>>> for Error {
    fn from(
        err: PoisonError<std::sync::MutexGuard<'_, HashMap<std::string::String, usize>>>,
    ) -> Self {
        Self::State(err.to_string())
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, Pool>>> for Error {
    fn from(err: PoisonError<std::sync::MutexGuard<Pool>>) -> Self {
        Self::State(err.to_string())
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, PathBuf>>> for Error {
    fn from(err: PoisonError<std::sync::MutexGuard<PathBuf>>) -> Self {
        Self::SSH(ssh::Error::App(err.into()))
    }
}

impl From<openssh::Error> for Error {
    fn from(err: openssh::Error) -> Self {
        Self::SSH(ssh::Error::from(err))
    }
}
impl From<storage::Error> for Error {
    fn from(e: storage::Error) -> Self {
        Self::Storage(e)
    }
}

impl From<commands::Error> for Error {
    fn from(e: commands::Error) -> Self {
        Self::Command(e)
    }
}

impl From<jobs::Error> for Error {
    fn from(e: jobs::Error) -> Self {
        Self::Job(e)
    }
}

impl From<PoisonError<MutexGuard<'_, Option<AppConfig>>>> for Error {
    fn from(e: PoisonError<MutexGuard<Option<AppConfig>>>) -> Self {
        Self::State(e.to_string())
    }
}

impl From<ssh::Error> for Error {
    fn from(err: ssh::Error) -> Self {
        Self::SSH(err)
    }
}

impl From<InquireError> for Error {
    fn from(err: InquireError) -> Self {
        Self::Inquire(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
fn main() {
    let os = env::consts::OS;
    let user = env::var_os("USER").expect(USER_PANIC);
    let args = env::args().collect::<Vec<String>>();

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
        env::set_var(
            "APP_LOG_DIR",
            format!("{home_dir}/.config/{BUNDLE_IDENTIFIER}/logs"),
        )
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
        env::set_var(
            "APP_LOG_DIR",
            format!("{home_dir}/Library/Logs/{BUNDLE_IDENTIFIER}"),
        )
    } else {
        panic!("Unsupported OS");
    }

    let cache_dir = env::var("APP_CACHE_DIR").expect("APP_CACHE_DIR not set");
    let config_dir = env::var("APP_CONFIG_DIR").expect("APP_CONFIG_DIR not set");
    let data_dir = env::var("APP_DATA_DIR").expect("APP_DATA_DIR not set");
    let log_dir = env::var("APP_LOG_DIR").expect("APP_LOG_DIR not set");
    let daemon_dir = format!("{cache_dir}/daemon");

    if !directory_exists(&cache_dir) {
        create_directory(&cache_dir).expect("could not create cache directory");
    }
    if !directory_exists(&config_dir) {
        create_directory(&config_dir).expect("could not create config directory");
    }
    if !directory_exists(&data_dir) {
        create_directory(&data_dir).expect("could not create data directory");
    }
    if !directory_exists(&log_dir) {
        create_directory(&log_dir).expect("could not create log directory");
    }
    if !directory_exists(&daemon_dir) {
        create_directory(&daemon_dir).expect("could not create daemon directory");
    }

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(format!("{log_dir}/bmu_cli.log"))
        .expect("could not create log file");

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Warn))
        .expect("could not create log config");

    log4rs::init_config(config).expect("could not init log4rs");

    match args.get(1) {
        Some(arg) => match arg.as_str() {
            "daemon" => handle_daemon(args),
            "clean" => {
                let storage = storage::Storage::load().expect("Could not load storage");
                let directories = jobs::maintenance::Directories {
                    cache: storage.cache_dir,
                    log: storage.log_dir,
                };
                let options = jobs::maintenance::Options {
                    connections: true,
                    daemon: true,
                    logs: true,
                };
                match jobs::maintenance::clean(directories, Some(options)) {
                    Err(why) => {
                        panic!("⛔️ Could not perform cleaning job {why:?}");
                    }
                    Ok(_) => println!("Done"),
                };
            }
            "help" => help(),
            _ => panic!("⛔️ Invalid argument '{arg}'"),
        },
        None => {
            let messages = vec![
                format!("USER: {}", user.to_str().expect(USER_PANIC)),
                format!("APP_CACHE_DIR: {cache_dir}"),
                format!("APP_CONFIG_DIR: {config_dir}"),
                format!("APP_DATA_DIR: {data_dir}"),
                format!("APP_LOG_DIR: {log_dir}"),
            ];
            menu::ui::print_frame(TITLE, messages, true);
            menu::show()
        }
    }
}

fn help() {
    let messages = vec![
        format!("To start the interactive menu: bmu\n"),
        format!("Other usage: bmu [daemon|clean|help]"),
        format!("{:10} {:22}", "  daemon", "[start|restart|stop]",),
        format!(
            "{:10} {:22} -- {}",
            "   start", "", "Starts background backups in a daemon"
        ),
        format!(
            "{:10} {:22} -- {}",
            "   stop", "", "Stops the deamon if its running"
        ),
        format!(
            "{:10} {:22} -- {}",
            "   restart", "", "Restarts the deamon if its running"
        ),
        format!(
            "{:10} {:22} -- {}",
            "  clean", "", "Clean the cache and logs"
        ),
        format!("{:10} {:22} -- {}", "  help", "", "Show this help message"),
    ];

    menu::ui::print_frame(TITLE, messages, true);
}

fn handle_daemon(args: Vec<String>) {
    let arg = args.get(2).expect("⛔️ Missing daemon argument");
    match arg.as_str() {
        "start" => daemon::start(),
        "restart" => daemon::restart(),
        "stop" => daemon::stop(),
        _ => panic!("⛔️ Invalid argument '{arg}'"),
    }
}

pub async fn set_state_and_test_connection(
    state: &MutexState,
    config: AppConfig,
) -> Result<AppConfig, Error> {
    if let Some(connection) = state.connection.lock().await.take() {
        connection.sftp_client.close().await?;
        connection.ssh_session.close().await?;
    };

    _ = state.config.lock()?.insert(config.clone());
    let app_cache_dir = state.app_cache_dir.lock()?.clone();
    let connection = ui::loader(
        "Testing connection...",
        Connection::new(config.clone(), app_cache_dir),
    )
    .await?;
    state.connection.lock().await.get_or_insert(connection);

    Ok(config)
}

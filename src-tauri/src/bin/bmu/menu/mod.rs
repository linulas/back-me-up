use super::storage;
use crate::{daemon, set_state_and_test_connection, Error};
use back_me_up::models::app::MutexState;
use back_me_up::{commands, jobs};
use inquire::InquireError;
use std::process;
use std::sync::{Arc, Mutex};

mod home;
mod settings;
mod setup;
pub mod ui;

pub enum Action {
    Show,
    Disconnect,
    Exit,
}

pub fn handle_inquire_error(err: InquireError) {
    match err {
        InquireError::OperationCanceled => (),
        InquireError::OperationInterrupted => process::exit(0),
        _ => println!("\n⛔️ Error selecting option: {err}"),
    };
}

#[tokio::main]
pub async fn show() {
    let storage = match storage::Storage::load() {
        Ok(s) => s,
        Err(why) => {
            println!("{why:?}");
            panic!("⛔️ Could not load storage");
        }
    };

    let pool = jobs::Pool::new(None);
    let state = MutexState {
        config: Mutex::default(),
        connection: tokio::sync::Mutex::default(),
        jobs: Arc::new(Mutex::default()),
        failed_jobs: Arc::new(Mutex::default()),
        pool: Mutex::new(pool),
        app_cache_dir: Arc::new(Mutex::new(storage.cache_dir.clone())),
        app_log_dir: Arc::new(Mutex::new(storage.log_dir.clone())),
    };

    let config = if let Some(c) = storage.config() {
        c
    } else {
        match setup::begin(&state).await {
            Ok(c) => c,
            Err(why) => {
                panic!("⛔️ Could not load config: {why:?}");
            }
        }
    };

    if state
        .config
        .lock()
        .expect("failed to lock config")
        .is_none()
        || state.connection.lock().await.is_none()
    {
        ui::loader(
            "Connecting...",
            set_state_and_test_connection(&state, config.clone()),
        )
        .await
        .expect("Failed to connect to server");
    }

    println!("✅ Connection successfull!\n");

    if config.allow_background_backup && !daemon::is_running(&storage) {
        commands::app::start_background_backups(
            &state,
            storage.backups().expect("could not load backups"),
        )
        .expect("could not start background backups");
    }

    loop {
        if let Err(why) = home::show(&state).await {
            match why {
                Error::Inquire(err) => handle_inquire_error(err),
                _ => println!("\n⛔️ Error: {why:?}\n"),
            }
        }
    }
}

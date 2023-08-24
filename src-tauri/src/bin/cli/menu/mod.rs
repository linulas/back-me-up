use super::storage;
use bmu::jobs;
use bmu::models::app::MutexState;
use std::sync::{Arc, Mutex};

mod home;
mod setup;
pub mod ui;

pub enum Action {
    Show,
    Exit,
}

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

    let pattern = format!("{}/.ssh-connection*", storage.cache_dir.display());
    bmu::jobs::fs::cleanup_entities_by_pattern(&pattern).expect("could not cleanup_connections");

    if state
        .config
        .lock()
        .expect("failed to lock config")
        .is_none()
        || state.connection.lock().await.is_none()
    {
        ui::loader(
            "Connecting...",
            setup::set_state_and_test_connection(&state, config),
        )
        .await
        .expect("Failed to connect to server");
    }

    println!("✅ Connection successfull!\n");

    loop {
        if let Err(why) = home::show(&state).await {
            println!("\n⛔️ Error: {why:?}\n");
        }
    }
}

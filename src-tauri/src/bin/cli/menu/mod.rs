use super::storage;
use bmu::jobs;
use bmu::models::app::MutexState;
use std::sync::{Arc, Mutex};

mod home;
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

    let config = if let Some(config) = storage.config() {
        config
    } else {
        todo!(); // TODO: implement setup menu
    };

    let pattern = format!("{}/.ssh-connection*", storage.cache_dir.display());
    bmu::jobs::fs::cleanup_entities_by_pattern(&pattern).expect("could not cleanup_connections");
    let pool = jobs::Pool::new(None);
    let connection = ui::loader(
        "Connecting...",
        bmu::ssh::connect::Connection::new(config.clone(), storage.cache_dir),
    )
    .await
    .expect("Failed to connect to server");
    println!("✅ Connection successfull!\n");

    let state = MutexState {
        config: Mutex::new(Some(config)),
        connection: tokio::sync::Mutex::new(Some(connection)),
        jobs: Arc::new(Mutex::default()),
        failed_jobs: Arc::new(Mutex::default()),
        pool: Mutex::new(pool),
        app_cache_dir: Arc::new(Mutex::default()),
    };

    loop {
        if let Err(why) = home::show(&state).await {
            println!("\n⛔️ Error: {why:?}\n");
        }
    }
}

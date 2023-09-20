use crate::storage::Storage;
use crate::{set_state_and_test_connection, storage};
use back_me_up::graceful_exit;
use back_me_up::models::app::MutexState;
use back_me_up::{commands, jobs};
use daemonize::Daemonize;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{fs, io, process};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let storage = storage::Storage::load().expect("⛔️ Could not load storage: {why:?}");
    fs::write(storage.daemon_dir.join("state"), "running")
        .expect("Could not write daemon state file");
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
    let config = storage
        .config()
        .expect("No config detected, please run 'bmu_cli' to setup");

    match set_state_and_test_connection(&state, config.clone()).await {
        Ok(_) => println!("✅ Connection successfull!\n"),
        Err(why) => {
            panic!("⛔️ Could not connect to server: {why:?}");
        }
    };

    commands::app::start_background_backups(
        &state,
        storage.backups().expect("could not load backups"),
    )
    .expect("could not start background backups");

    loop {
        let action = fs::read_to_string(format!("{}/state", storage.daemon_dir.display()))
            .unwrap_or_default();

        if action.trim() == "terminate" {
            break;
        };

        sleep(Duration::from_secs(5)).await;
    }

    fs::write(storage.daemon_dir.join("state"), "stopped").unwrap_or_default();
    graceful_exit(&state).await;
}

pub fn start() {
    let storage = storage::Storage::load().expect("⛔️ Could not load storage: {why:?}\nYou might need to initialize setup by running 'bmu_cli' first");
    if is_running(&storage) {
        println!("Daemon is already running");
        process::exit(0);
    }
    let daemon_dir = storage.daemon_dir.display().to_string();
    let stdout =
        File::create(format!("{daemon_dir}/bmu_cli.out")).expect("could not create daemon.out");
    let stderr =
        File::create(format!("{daemon_dir}/bmu_cli.err")).expect("could not create daemon.err");

    let daemonize = Daemonize::new()
        .pid_file(format!("{daemon_dir}/bmu_cli.pid"))
        .chown_pid_file(true)
        .working_directory(format!("{daemon_dir}"))
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    let config = storage
        .config()
        .expect("No config detected, please run 'bmu_cli' to setup");

    if !config.allow_background_backup {
        panic!("⛔️ Background backups are disabled, please run 'bmu_cli' and go into settings to enable them");
    }

    crate::menu::ui::print_frame(
        "Back me up 🚀",
        vec![
            String::from("⚙️  Jobs are running in the background"),
            format!("👉 Output can be found in: {daemon_dir}"),
            String::from("💻 Run 'bmu daemon stop' to stop the daemon"),
        ],
        false,
    );

    if let Err(e) = daemonize.start() {
        panic!("⛔️ Error starting daemon: {e:?}");
    } else {
        main();
    }
}

pub fn stop() {
    let storage = Storage::load().expect("could not load storage");
    let daemon_dir = storage.daemon_dir.display();
    let current_state = fs::read_to_string(storage.daemon_dir.join("state"))
        .expect("Could not read daemon state file");

    if current_state.trim() != "running" {
        println!("Daemon is already stopped");
        process::exit(0);
    }

    fs::write(storage.daemon_dir.join("state"), "terminate")
        .expect("Could not write daemon state file");

    crate::menu::ui::print_frame(
        "Back Me Up",
        vec![
            "Terminate message was sent to the daemon".to_string(),
            format!("Check '{daemon_dir}' to verify that it closed as expected"),
        ],
        true,
    );
}

pub fn restart() {
    let storage = storage::Storage::load().expect("could not load storage");
    stop();
    let mut try_count = 0;
    while is_terminating(&storage) && try_count < 10 {
        try_count += 1;
        print!("\r🔄 Checking if daemon has stopped. Try count: {try_count}");
        io::stdout().flush().expect("could not flush stdout");
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!("\r\x1B[1A\x1B[2K");
        io::stdout().flush().expect("failed to flush stdout");
    }

    println!("{}", " ".repeat(50));

    if !is_stopped(&storage) {
        panic!("⛔️ Could not restart daemon");
    }

    start();
}

pub fn is_running(storage: &Storage) -> bool {
    let state = fs::read_to_string(storage.daemon_dir.join("state")).unwrap_or_default();
    state.trim() == "running"
}

pub fn is_terminating(storage: &Storage) -> bool {
    let state = fs::read_to_string(storage.daemon_dir.join("state")).unwrap_or_default();
    state.trim() == "terminate"
}

pub fn is_stopped(storage: &Storage) -> bool {
    let state = fs::read_to_string(storage.daemon_dir.join("state")).unwrap_or_default();
    state.trim() == "stopped"
}

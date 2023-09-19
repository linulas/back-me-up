use log::error;
use std::io::{self, Write};

pub mod commands;
pub mod jobs;
pub mod models;
pub mod ssh;

#[cfg(test)]
mod tests;

pub async fn graceful_exit(state: &models::app::MutexState) {
    if let Some(connection) = state.connection.lock().await.take() {
        if let Err(e) = connection.sftp_client.close().await {
            error!("⛔️ Could not disconnect sftp client: {e:?}");
            println!("⛔️ Could not disconnect sftp client\n");
        } else {
            println!("🔌 Disconnected sftp client    \n"); // add trailing balnkspace to overwrite previous loading output
        };

        if let Err(e) = connection.ssh_session.close().await {
            error!("⛔️ Could not close ssh session: {e:?}");
            println!("⛔️ Could not close ssh session\n");
        } else {
            println!("🔌 Disconnected ssh session\n");
        }
    };

    let mut jobs = state.jobs.lock().expect("could not lock jobs");
    let mut pool = state.pool.lock().expect("could not lock pool");
    let total_jobs_to_terminate = jobs.len().clone();

    print!("\r⏳ Terminating jobs...      "); // add trailing blankspaces to overwrite previous loading output
    io::stdout().flush().expect("failed to flush stdout");
    jobs::backup::terminate_all(&mut jobs, &mut pool);
    println!("\r\x1B[1A\x1B[2K");
    io::stdout().flush().expect("failed to flush stdout");

    if jobs.len() > 0 {
        println!("⛔️ Not all jobs could be terminated\n");
        println!(
            "   Sucessfully terminated: {}",
            total_jobs_to_terminate - jobs.len()
        );
        println!("  Failed to terminate: {}", jobs.len());
    } else {
        println!("✋ Stopped all ongoing jobs\n");
    }

    println!("✅ Done\n");

    std::process::exit(0);
}

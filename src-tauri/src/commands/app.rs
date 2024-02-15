use super::{os, Error};
use crate::jobs;
use crate::models::app::{self, MutexState};
use crate::models::backup::{Backup, Kind as BackupKind};
use log::{error, info};
use std::path::PathBuf;
use std::sync::Arc;

pub fn backup_on_change(state: &MutexState, backup: Backup) -> Result<(), Error> {
    let state_config = &state.config.lock()?;
    let config_to_move_into_thread = if let Some(config) = state_config.as_ref() {
        config.clone()
    } else {
        let error = app::Error::Config(String::from("No config detected"));
        return Err(Error::App(error));
    };

    if !config_to_move_into_thread.allow_background_backup {
        return Err(Error::App(app::Error::Config(String::from(
            "Background backup is disabled",
        ))));
    }

    let job_id = jobs::id_from_backup(&backup, &jobs::Kind::BackupFolderOnChange);
    let jobs = Arc::clone(&state.jobs);

    if jobs.lock()?.iter().any(|(id, _)| id == &job_id) {
        info!(
            "Already running background backup for {}",
            backup.client_location.path
        );
        return Ok(());
    };

    let mut pool = state.pool.lock()?;
    pool.execute(move |worker| {
        jobs.lock()
            .expect("Could not lock jobs")
            .insert(job_id, worker.id);
        jobs::backup::entity_on_change(&worker, &backup, config_to_move_into_thread);
    })?;

    Ok(())
}

pub fn terminate_background_backup(state: &MutexState, backup: &Backup) -> Result<(), Error> {
    let mut jobs = state.jobs.lock()?;
    let job_id = &jobs::id_from_backup(backup, &jobs::Kind::BackupFolderOnChange);
    let worker_id = if let Some(id) = jobs.get(job_id) {
        id
    } else {
        let error = jobs::Error::NotFound("could not find job".to_string());
        return Err(Error::Job(error));
    };

    let mut pool = state.pool.lock()?;
    let result = pool.terminate_job(*worker_id, || {
        let path_buf = PathBuf::from(&backup.client_location.path);
        if !path_buf.exists() {
            error!("File does not exist: {path_buf:?}");
            return;
        }

        let client_entity_path = &backup.client_location.path;

        let trigger_file_path = match BackupKind::from(&path_buf) {
            BackupKind::File => {
                let parent = path_buf
                    .parent()
                    .expect("File should have a parent")
                    .display()
                    .to_string();
                format!("{parent}/.bmu_event_trigger")
            }
            BackupKind::Directory => {
                format!("{client_entity_path}/.bmu_event_trigger")
            }
        };

        if let Err(e) = os::create_file(&trigger_file_path) {
            error!("Failed to create file: {e:?}");
        }

        if let Err(e) = os::delete_file(&trigger_file_path) {
            error!("Failed to delete file: {e:?}");
        }
    });

    if let Err(e) = result {
        let error = jobs::Error::Terminate(e);
        return Err(Error::Job(error));
    }

    if jobs.remove(job_id).is_some() {
        Ok(())
    } else {
        let error = jobs::Error::Terminate(String::from(
            "Worker was terminated, but couldn't remove job from active list",
        ));
        Err(Error::Job(error))
    }
}

pub fn start_background_backups(state: &MutexState, backups: &[Backup]) -> Result<(), Error> {
    let state_config = &state.config.lock()?;

    if let Some(config) = state_config.as_ref() {
        if !config.allow_background_backup {
            return Err(Error::App(app::Error::Config(String::from(
                "Background backup is disabled",
            ))));
        }
    }

    let jobs = state.jobs.lock()?;
    let available_workers = state.pool.lock()?.available_workers();
    let backup_jobs_that_are_not_already_running: Vec<_> = backups
        .iter()
        .filter(|b| {
            !jobs.iter().any(|(job_id, _)| {
                job_id == &jobs::id_from_backup(b, &jobs::Kind::BackupFolderOnChange)
            })
        })
        .collect();

    let num_of_backups_to_start = backup_jobs_that_are_not_already_running.len();

    if num_of_backups_to_start > available_workers {
        state
            .pool
            .lock()?
            .create_workers(num_of_backups_to_start - available_workers);
    }

    if num_of_backups_to_start == 0 {
        return Ok(());
    }

    state.pool.lock()?.start_all_stopped_workers();

    for value in backup_jobs_that_are_not_already_running {
        let config_to_move_into_thread = if let Some(config) = state_config.as_ref() {
            config.clone()
        } else {
            let error = app::Error::Config(String::from("No config detected"));
            return Err(Error::App(error));
        };

        let backup = value.clone();

        let job_id = jobs::id_from_backup(&backup, &jobs::Kind::BackupFolderOnChange);
        let jobs = Arc::clone(&state.jobs);

        let mut pool = state.pool.lock()?;
        pool.execute(move |worker| {
            jobs.lock()
                .expect("Could not lock jobs")
                .insert(job_id, worker.id);
            jobs::backup::entity_on_change(&worker, &backup, config_to_move_into_thread);
        })?;
    }

    Ok(())
}

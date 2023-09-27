use super::Error;
use crate::jobs;
use crate::models::app::{self, MutexState};
use crate::models::backup::Backup;
use log::info;
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

    let job_id = jobs::id_from_backup(&backup, &jobs::Kind::BackupOnChange);
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
        jobs::backup::directory_on_change(&worker, &backup, config_to_move_into_thread);
    })?;

    Ok(())
}

pub fn terminate_background_backup(state: &MutexState, backup: &Backup) -> Result<(), Error> {
    let mut jobs = state.jobs.lock()?;
    let job_id = &jobs::id_from_backup(backup, &jobs::Kind::BackupOnChange);
    let worker_id = if let Some(id) = jobs.get(job_id) {
        id
    } else {
        let error = jobs::Error::NotFound("could not find job".to_string());
        return Err(Error::Job(error));
    };

    let mut pool = state.pool.lock()?;
    let result = pool.terminate_job(*worker_id, || {
        let file_path = format!("{}/.bmu_event_trigger", backup.client_location.path);

        _ = super::os::create_file(&file_path);
        _ = super::os::delete_file(&file_path);
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
            !jobs
                .iter()
                .any(|(job_id, _)| job_id == &jobs::id_from_backup(b, &jobs::Kind::BackupOnChange))
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

        let job_id = jobs::id_from_backup(&backup, &jobs::Kind::BackupOnChange);
        let jobs = Arc::clone(&state.jobs);

        let mut pool = state.pool.lock()?;
        pool.execute(move |worker| {
            jobs.lock()
                .expect("Could not lock jobs")
                .insert(job_id, worker.id);
            jobs::backup::directory_on_change(&worker, &backup, config_to_move_into_thread);
        })?;
    }

    Ok(())
}

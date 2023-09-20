use crate::models::app::MutexState;

use super::Error;

pub struct Options {
    pub connections: bool,
    pub daemon: bool,
    pub logs: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            connections: false,
            daemon: false,
            logs: true,
        }
    }
}

pub fn clean(state: &MutexState, opts: Option<Options>) -> Result<(), Error> {
    let options = opts.unwrap_or_default();
    let cache_dir = state.app_cache_dir.lock()?.clone();

    if options.connections {
        let pattern = format!("{}/.ssh-connection*", cache_dir.display());
        super::fs::cleanup_entities_by_pattern(&pattern)?;
    }

    if options.daemon {
        let pattern = format!("{}/daemon/*", cache_dir.display());
        super::fs::cleanup_entities_by_pattern(&pattern)?;
    }

    if options.logs {
        let log_dir = state.app_log_dir.lock()?.clone();
        let pattern = format!("{}/*.log", log_dir.display());
        super::fs::cleanup_entities_by_pattern(&pattern)?;
    }

    Ok(())
}

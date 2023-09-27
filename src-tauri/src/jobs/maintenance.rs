use std::path::PathBuf;

use super::Error;

pub struct Options {
    pub connections: bool,
    pub daemon: bool,
    pub logs: bool,
}

pub struct Directories {
    pub cache: PathBuf,
    pub log: PathBuf,
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

pub fn clean(directories: &Directories, opts: Option<Options>) -> Result<(), Error> {
    let options = opts.unwrap_or_default();

    if options.connections {
        let pattern = format!("{}/.ssh-connection*", directories.cache.display());
        super::fs::cleanup_entities_by_pattern(&pattern)?;
    }

    if options.daemon {
        let pattern = format!("{}/daemon/*", directories.cache.display());
        super::fs::cleanup_entities_by_pattern(&pattern)?;
    }

    if options.logs {
        let pattern = format!("{}/*.log", directories.log.display());
        super::fs::cleanup_entities_by_pattern(&pattern)?;
    }

    Ok(())
}

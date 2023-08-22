use std::fs;
use glob::{glob, PatternError};
use super::Error;

impl From<PatternError> for Error {
    fn from(e: PatternError) -> Self {
        Self::Pattern(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Command(e.to_string())
    }
}

pub fn cleanup_entities_by_pattern(pattern: &str) -> Result<(), Error> {
    for path in (glob(pattern)?).flatten() {
        if path.is_file() {
            fs::remove_file(path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
        }
    }

    log::info!("cleanup successfull!");
    Ok(())
}

use std::path::PathBuf;
use std::{env, fs};

use bmu::models::app::Config;
use bmu::models::backup::Backup;

#[derive(Debug)]
pub enum Error {
    NotUnique(String),
    Env(env::VarError),
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Self {
        Self::Env(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

pub struct Storage {
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Storage {
    pub fn load() -> Result<Self, Error> {
        let cache_dir = PathBuf::from(env::var("APP_CACHE_DIR")?);
        let config_dir = PathBuf::from(env::var("APP_CONFIG_DIR")?);
        let data_dir = PathBuf::from(env::var("APP_DATA_DIR")?);

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }
        if !cache_dir.is_dir() {
            fs::create_dir_all(&cache_dir)?;
        }
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        if !config_dir.is_dir() {
            panic!("APP_CONFIG_DIR is not a directory");
        }
        if !data_dir.exists() {
            panic!("APP_DATA_DIR does not exist");
        }
        if !data_dir.is_dir() {
            panic!("APP_DATA_DIR is not a directory");
        }

        Ok(Self {
            cache_dir,
            config_dir,
            data_dir,
        })
    }

    pub fn config(&self) -> Option<Config> {
        if !self.config_dir.join("server.conf.json").exists() {
            return None;
        }

        let config_file_path = self.config_dir.join("server.conf.json");
        let config_file_contents =
            std::fs::read_to_string(&config_file_path).expect("Failed to read config file");

        Some(serde_json::from_str(&config_file_contents).expect("Failed to parse config file"))
    }

    pub fn write_conig(&self, config: Config) {
        let config_file_path = self.config_dir.join("server.conf.json");
        let config_file_contents =
            serde_json::to_string(&config).expect("Failed to serialize config");
        std::fs::write(&config_file_path, config_file_contents)
            .expect("Failed to write config file");
    }

    pub fn backups(&self) -> Result<Vec<Backup>, Error> {
        if !self.data_dir.join("backups.json").exists() {
            return Ok(vec![]);
        }

        let backup_file_path = self.data_dir.join("backups.json");
        let backup_file_contents = std::fs::read_to_string(&backup_file_path)?;

        Ok(serde_json::from_str(&backup_file_contents)?)
    }

    pub fn add_backup(&self, backup: Backup) -> Result<(), Error> {
        let mut backups = self.backups()?;
        if backups.iter().any(|b| {
            b.client_location.path == backup.client_location.path
                && b.server_location.path == backup.server_location.path
        }) {
            return Err(Error::NotUnique("Backup already exists".to_string()));
        };
        backups.push(backup);
        self.write_backups(backups);

        Ok(())
    }

    pub fn delete_backup(&self, backup: Backup) -> Result<(), Error> {
        let mut backups = self.backups()?;
        backups.retain(|b| {
            b.client_location.path != backup.client_location.path
                || b.server_location.path != backup.server_location.path
        });
        self.write_backups(backups);

        Ok(())
    }

    pub fn reset(&self) -> Result<(), Error> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
        }
        if self.config_dir.exists() {
            fs::remove_dir_all(&self.config_dir)?;
        }
        if self.data_dir.exists() {
            fs::remove_dir_all(&self.data_dir)?;
        }

        Ok(())
    }

    fn write_backups(&self, backups: Vec<Backup>) {
        let backup_file_path = self.data_dir.join("backups.json");
        let backup_file_contents =
            serde_json::to_string(&backups).expect("Failed to serialize backup file");
        std::fs::write(&backup_file_path, backup_file_contents)
            .expect("Failed to write backup file");
    }
}

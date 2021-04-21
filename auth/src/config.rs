use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub username: String,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TomlParseError(toml::de::Error),
    TomlSerError(toml::ser::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IoError(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Error {
        Error::TomlParseError(error)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(error: toml::ser::Error) -> Error {
        Error::TomlSerError(error)
    }
}

fn config_path() -> PathBuf {
    let mut config_path = dirs::config_dir().unwrap();
    config_path.push("gitbucket.toml");
    config_path
}

pub fn load_config() -> Option<Config> {
    let mut config_data = vec![];

    let mut io = File::open(config_path()).ok()?;
    io.read_to_end(&mut config_data).ok()?;

    toml::from_slice(&config_data).ok()
}

pub fn save_config(config: &Config) -> Result<(), Error> {
    let mut io = File::create(config_path())?;
    let config_data = toml::to_vec(config)?;

    io.write_all(&config_data)?;

    Ok(())
}

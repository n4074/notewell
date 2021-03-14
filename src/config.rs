 use serde::Deserialize;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use std::path::{PathBuf, Path};
use log::{warn};

#[derive(Deserialize)]
pub struct Config {
    pub index_dir: PathBuf,
    pub notes_dir: PathBuf,
}

impl Config {
    fn new(index_dir: &str, notes_dir: &str) -> Result<Config> {
        return Ok(Config {
            index_dir: Path::new(index_dir).to_owned(),
            notes_dir: Path::new(notes_dir).to_owned(),
        });
    }

    fn default() -> Config {
        return Config {
            index_dir: Path::new("/tmp/index").to_owned(),
            notes_dir: Path::new("/tmp/notes").to_owned(),
        };
    }
}

/// Attempt to load and parse the config file into our Config struct.
/// If a file cannot be found, return a default Config.
/// If we find a file but cannot parse it, panic
pub fn parse(path: &str) -> anyhow::Result<Config> {
    let mut config_toml = String::new();

    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => {
            warn!("Could not find config file, using default!");
            return Ok(Config::default());
        }
    };
    // let file = File::open(&path)
    //     .with_context(|| format!("Error while opening config {}", path))?;
    file.read_to_string(&mut config_toml)
        .with_context(|| format!("Error while reading config {}", path))?;
    return toml::from_str(&config_toml).context("Failed to decode config file");
}
 use serde::Deserialize;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use log::{warn};

#[derive(Deserialize)]
pub struct Config {
    pub index_dir: String,
    pub notes_dir: String,
}

impl Config {
    fn new(index_dir: &str, notes_dir: &str) -> Result<Config> {
        return Ok(Config {
            index_dir: index_dir.to_string(),
            notes_dir: notes_dir.to_string(),
        });
    }

    fn default() -> Config {
        return Config {
            index_dir: "/tmp/index".to_string(),
            notes_dir: "/tmp/notes".to_string(),
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
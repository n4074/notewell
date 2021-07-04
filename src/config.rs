 use serde::Deserialize;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use std::path::{PathBuf, Path};
use log::{warn};

#[derive(Deserialize)]
pub struct Config {
    pub state: PathBuf,
    pub index: PathBuf,
    pub notes: PathBuf,
}

impl Config {
    #[allow(dead_code)]
    fn new(index: &str, notes: &str, state: &str) -> Result<Config> {
        return Ok(Config {
            state: Path::new(state).to_owned(),
            index: Path::new(index).to_owned(),
            notes: Path::new(notes).to_owned(),
        });
    }

    fn default() -> Config {
        return Config {
            state: Path::new("/tmp/nb").to_owned(),
            index: Path::new("/tmp/nb/index").to_owned(),
            notes: Path::new("/tmp/notes").to_owned(),
        };
    }

    /// Attempt to load and parse the config file into our Config struct.
    /// If a file cannot be found, return a default Config.
    /// If we find a file but cannot parse it, panic
    pub fn open_or_create<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
        let mut config_toml = String::new();

        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => {
                warn!("Could not find config file, using embedded default");
                // TODO: Generate config file
                return Ok(Config::default());
            }
        };
        // let file = File::open(&path)
        //     .with_context(|| format!("Error while opening config {}", path))?;
        file.read_to_string(&mut config_toml)
            .with_context(|| format!("Error while reading config {}", path.as_ref().to_string_lossy()))?;
        return toml::from_str(&config_toml).context("Failed to decode config file");
    }

}
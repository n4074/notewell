// #[macro_use]

use log::{debug, error, info, warn};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{PathBuf, Path};

use git2::Oid;
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};

use derive_new::new;
use clap::Arg;

use pulldown_cmark::{Event, Options, Parser, Tag};

mod index;
mod git;
mod config;

use git::*;
use index::*;

fn parse_args() -> String {
    let matches = clap::App::new("noteater")
        .version("0.1")
        .author("Carl Hattenfels")
        .about("Simple note search interface")
        .arg(
            Arg::with_name("QUERYSTRING")
                .help("Query to run against notes directory")
                .required(true)
                .index(1),
        )
        .get_matches();
    return String::from(matches.value_of("QUERYSTRING").unwrap());
}

#[derive(Serialize, Deserialize, Default)]
struct State {
    commit: Option<String>, // Last index commit
    #[serde(skip)]
    path: PathBuf 
}

impl State {
    fn open_or_create<P: AsRef<Path>>(path: P) -> Result<State> {
        if !path.as_ref().to_owned().exists() {
            let mut state: State = Default::default();
            state.path = path.as_ref().to_owned();
            state.save()?;
            Ok(state)
        } else {
            let file = File::open(&path)?;
            let mut state: State = serde_json::from_reader(file)
                .context("Failed to deserialize state")?;
            state.path = path.as_ref().to_owned();
            Ok(state)
        }
    }

    fn save(&self) -> Result<()> {
        let file = std::fs::OpenOptions::new().write(true).truncate(true).open(&self.path)?;
        serde_json::to_writer(file, &self).context("Failed to serialise state")
    }
}

struct App {
    pub appdir: PathBuf,
    pub config: config::Config,
    pub index: index::Index,
    pub repo: git::Repo, 
    pub state: State,
}

impl App {
    fn new(appdir: PathBuf) -> Result<App> {
        let config = config::Config::open_or_create(appdir.clone().join("config.toml"))?;
        let repo = Repo::open_or_create(&config.notes)?;
        let index = Index::open_or_create(&config.index)?;
        let state = State::open_or_create(appdir.clone().join("state"))?;

        Ok(App {
            config,
            index,
            repo,
            state,
            appdir
        })
    }

    fn sync(&mut self) -> anyhow::Result<()> {

        let head = self.repo.head()?;
        let diffs = self.repo.diff(self.state.commit.as_ref(), None)?;
    
        for diff in diffs {
            match diff {
                (git2::Delta::Added, path) | (git2::Delta::Modified, path) => { 
                    self.index.delete(&path);

                    let mut doc = self.index.documentbuilder(&path);

                    let content = std::fs::read_to_string(&self.config.notes.join(&doc.path))?;
                    doc.body(&content);

                    self.index.add(&path, doc.document());
                }
                (git2::Delta::Deleted, path) => { self.index.delete(&path) } 
                (git2::Delta::Renamed, path) => { todo!("Handling Renaming. Need both old and new path") } 
                _ => todo!()
            }
        } 

        self.index.commit()?;
        self.index.reload()?;

        self.state.commit = Some(head.id().to_string());
        self.state.save()?;


        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let query = parse_args();
    let mut app = App::new(PathBuf::new())?;

    app.sync()?;

    app.index.query(&query)?;

    app.state.save()?;

    return Ok(());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
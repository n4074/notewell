// #[macro_use]

use log::{debug, info};
use std::fs::File;
use std::path::{PathBuf, Path};
//use std::ffi::OsStr;

use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};

use std::process::Command; 

use clap::Arg;


//use pulldown_cmark::{Event, Options, Parser, Tag};

mod index;
mod repo;
mod config;
mod heap;

use repo::*;
use index::*;

fn arg_parser<'a,'b>() -> clap::App<'a,'b> {
    return clap::App::new("noteater")
        .version("0.1")
        .author("Carl Hattenfels")
        .about("Simple note search interface")
        .subcommand(clap::SubCommand::with_name("search")
            .arg(
                Arg::with_name("QUERYSTRING")
                    .help("Query to run against notes directory")
                    .required(true)
                    //.last(true)
                    .index(1)
                    .multiple(true),
            )
        )
        .subcommand(clap::SubCommand::with_name("add")
            .about("add a new note")
            .arg(Arg::with_name("path")
                .short("p")
                .help("note path"))
        )
        .subcommand(clap::SubCommand::with_name("edit")
            .about("edit an existing note")
            .arg(Arg::with_name("PATH")
                .index(1)
                .help("note path"))
        )
}

fn new_note<P: AsRef<Path>>(path: P) {
    Command::new("vim")
        .args(&[path.as_ref()])
        .spawn()
        .expect("wat");
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
        debug!("Saving state to {}", self.path.to_string_lossy());
        let file = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&self.path)?;
        serde_json::to_writer(file, &self).context("Failed to serialise state")
    }
}

struct App {
    pub appdir: PathBuf,
    pub config: config::Config,
    pub index: index::Index,
    pub repo: repo::Repo, 
    pub state: State,
}

impl App {
    fn open_or_create<P: AsRef<Path>>(appdir: P) -> Result<App> {

        if !appdir.as_ref().exists() {
            info!("Creating new app directory at {}", appdir.as_ref().to_string_lossy());
            std::fs::create_dir_all(&appdir)?;
        }

        let config = config::Config::open_or_create(appdir.as_ref().clone().join("config.toml"))?;
        let state = State::open_or_create(appdir.as_ref().clone().join("state"))?;
        let repo = Repo::init(&config.notes)?;
        let index = Index::create(&config.index)?;

        Ok(App {
            config,
            index,
            repo,
            state,
            appdir: appdir.as_ref().to_path_buf()
        })
    }

    fn sync(&mut self) -> anyhow::Result<()> {

        let head = self.repo.head()?;
        let diffs = self.repo.diff(self.state.commit.as_ref(), None)?;
    
        for diff in diffs {
            match diff {
                (git2::Delta::Added, path) | (git2::Delta::Modified, path) => { 
                    self.index.delete(&path);
                    println!("{:?}", path);

                    let mut note = self.index.notebuilder(&path);

                    let content = std::fs::read_to_string(&self.config.notes.join(&note.path))?;
                    note.body(&content);

                    //let doc = doc_builder.document();

                    self.index.add(&path, note);
                }
                (git2::Delta::Deleted, path) => { self.index.delete(&path) } 
                (git2::Delta::Renamed, _path) => { todo!("Handling Renaming. Need both old and new path") } 
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
    env_logger::init();

    let app = arg_parser();

    //let query = String::from(matches.value_of("QUERYSTRING").unwrap());
    match app.clone().get_matches().subcommand() {
        ("edit", Some(path)) => { new_note(path.value_of("PATH").unwrap()) }
        ("init", Some(path)) => { }
        ("search", Some(args)) => {
            let query = String::from(args.value_of("QUERYSTRING").unwrap());
            println!("{:?}", query);
            //if let Ok(res) = app.index.query(&query) {
            //    println!("{:?}", res);
            //}
        }
        (command,_) => {
            app.clone().print_help();
            return Ok(());
        }
    }

    let appdir = std::env::var("NB").unwrap_or("~/.nb".to_owned());
    let mut app = App::open_or_create(appdir)?;

    app.sync()?;



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
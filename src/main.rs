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

//fn filecontent_to_body(index: &mut index::Index, path: &std::path::Path) -> Result<tantivy::schema::FieldValue> {
//    let value = tantivy::schema::Value::Str(std::fs::read_to_string(path)?);
//    let field = index.schema.get_field("path").unwrap();
//    Ok(tantivy::schema::FieldValue::new(field, value))
//}

//fn add_body<'a>(index: &mut index::Index, config: &config::Config, doc: &'a mut tantivy::Document) -> Result<&'a mut tantivy::Document> {
//    let pathfield = index.schema.get_field("path").context("could not get path field")?;
//    let bodyfield = index.schema.get_field("body").context("could not get body field")?;
//    let mut paths: Vec<String> = vec!();
//    for path in (&doc).get_all(pathfield) {
//        if let tantivy::schema::Value::Str(path) = path { 
//            paths.push(path.clone());
//        }   
//    }
//
//    for path in paths {
//        println!("{:?}", &path);
//        let content = std::fs::read_to_string(&config.notes.join(path))?;
//        doc.add_text(bodyfield, &content);
//        println!("Adding content: {:?}", &content);
//    }
//    
//    Ok(doc)
//}

fn sync(index: &mut index::Index, config: &config::Config, diffs: Vec<(git2::Delta, std::path::PathBuf)>) -> anyhow::Result<()> {
    //let text = index.schema.get_field("body").unwrap();
    //let mut ops = vec![];
   
    for diff in diffs {
        match diff {
            (git2::Delta::Added, path) | (git2::Delta::Modified, path) => { 
                index.delete(&path);

                let mut doc = index.documentbuilder(&path);

                let content = std::fs::read_to_string(&config.notes.join(&doc.path))?;
                doc.body(&content);

                index.add(&path, doc.document());
            }
            (git2::Delta::Deleted, path) => { index.delete(&path) } 
            (git2::Delta::Renamed, path) => { todo!("Handling Renaming. Need both old and new path") } 
            _ => todo!()
        }
    } 
    index.commit()?;
    Ok(())
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
            let file = File::create(&path);
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

        //let path: PathBuf = Default::default();

        //let commit: Option<Oid> = serde_json::from_reader(file)
        //    .context("Failed to read statefile")
        //    .and_then(|s: String| 
        //        git2::Oid::from_str(&s)
        //            .context("Failed to convert statefile")
        //    ).ok();

        //serde_json::from_reader(file)
        //    .context("Failed to deserialize state")?;

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
        //let commit = std::fs::read_to_string(appdir.join("commit"))
        //    .context("Failed to read statefile")
        //    .and_then(|s| 
        //        git2::Oid::from_str(&s)
        //            .context("Failed to convert statefile")
        //    ).ok();

        //let commit = Self::read_state(&appdir.join("state"))?;

        Ok(App {
            config,
            index,
            repo,
            state,
            appdir
        })
    }

    //fn read_state(path: &Path) -> Result<Option<String>> {
    //    let file = File::open(path)?;
    //    //let commit: Option<Oid> = serde_json::from_reader(file)
    //    //    .context("Failed to read statefile")
    //    //    .and_then(|s: String| 
    //    //        git2::Oid::from_str(&s)
    //    //            .context("Failed to convert statefile")
    //    //    ).ok();
    //    let commit: Option<String> = serde_json::from_reader(file)
    //        .context("Failed to deserialize state")?;

    //    if commit.is_some() && commit.unwrap().len() == 0 {
    //        Ok(None)
    //    } else {
    //        Ok(commit)
    //    } 
    //}

    //fn write_state(&self, path: &Path) -> Result<()> {
    //    let file = File::create(path)?;
    //    //let commit: String = self.commit.map(|oid| oid.to_string()).context("Could not convert oid")?;
    //    serde_json::to_writer(file, &self.commit).context("Failed to serialise commit")
    //}
}

fn main() -> anyhow::Result<()> {
    let query = parse_args();
    //let config = config::Config::open_or_create("./config.toml")?;

    //let statefile = std::fs::OpenOptions::new().read(true).write(true).create(true).open(&config.state)?;
    //let State { commit } = serde_json::from_reader(&statefile).unwrap_or(State { commit: None });

    //let repo = Repo::open_or_create(&config.notes)?;
    //let mut index = Index::open_or_create(&config.index)?;
    let mut app = App::new(PathBuf::new())?;

    let old = if let Some( oid ) = &app.state.commit {
        Some(app.repo.resolve(&oid)?.peel_to_commit()?)
    } else {
        None
    };

    let head = app.repo.head()?;
    let diffs = app.repo.diff(old, None)?;

    sync(&mut app.index, &app.config, diffs)?;

    //let statefile = std::fs::OpenOptions::new().truncate(true).write(true).create(true).open(&config.state)?;

    //serde_json::to_writer(&statefile, &State { commit })?;

    println!("Querying");
    app.index.reload()?;
    app.index.query(&query)?;

    app.state.commit = Some(head.id().to_string());
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
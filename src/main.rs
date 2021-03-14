// #[macro_use]

use log::{debug, error, info, warn};
use std::fs;
use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};

use derive_new::new;
use clap::{App, Arg};

use pulldown_cmark::{Event, Options, Parser, Tag};

mod index;
mod git;
mod config;

fn parse_args() -> String {
    let matches = App::new("noteater")
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

fn filecontent_to_body(index: &mut index::Index, path: &std::path::Path) -> Result<tantivy::schema::FieldValue> {
    let value = tantivy::schema::Value::Str(std::fs::read_to_string(path)?);
    let field = index.schema.get_field("path").unwrap();
    Ok(tantivy::schema::FieldValue::new(field, value))
}

fn sync(index: &mut index::Index, diffs: Vec<(git2::Delta, std::path::PathBuf)>) -> anyhow::Result<()> {
    let text = index.schema.get_field("body").unwrap();
    
    for diff in diffs {
        match diff {
            (git2::Delta::Added, path) | (git2::Delta::Modified, path) => { 
                //let content = std::fs::read_to_string(&path)?;
                let content = String::from("Here is some text to search one.");
                println!("{:?} {:?}", &path, &content);
                index.add(&path, vec!(tantivy::schema::FieldValue::new(text, tantivy::schema::Value::Str(content) )))?;
            }
            (git2::Delta::Deleted, path) => { index.delete(&path)? } 
            (git2::Delta::Renamed, path) => { todo!("Handling Renaming. Need both old and new path") } 
            _ => todo!()
        }
    } 
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct State {
    commit: Option<String>, // Last index commit
}

fn main() -> anyhow::Result<()> {
    let query = parse_args();
    let config = config::parse("./config.toml")?;
    let statefile = std::fs::OpenOptions::new().read(true).write(true).create(true).open(&config.state)?;

    let State { commit } = serde_json::from_reader(&statefile).unwrap_or(State { commit: None });
    let commit_oid = commit.map(|oid| git2::Oid::from_str(&oid).unwrap());
    println!("Commit Oid: {:?}", commit_oid);
    let mut nb = git::NoteWell::init(&config.notes, commit_oid)?;
    let (head, diffs) = nb.diff()?;
    println!("{:?}", head);
    let mut index = index::Index::open_or_create(&config.index)?;
    sync(&mut index, diffs)?;
    nb.synced(head)?;

    let statefile = std::fs::OpenOptions::new().truncate(true).write(true).create(true).open(&config.state)?;

    serde_json::to_writer(&statefile, &State { commit: Some(head.to_string()) })?;
    println!("Querying");
    index.reload()?;
    index.query(&query)?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
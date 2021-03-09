// #[macro_use]

use log::{debug, error, info, warn};
use std::fs;
use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result};
use serde::Deserialize;

use derive_new::new;
use clap::{App, Arg};

use pulldown_cmark::{Event, Options, Parser, Tag};

mod index;
mod git;
mod config;


pub trait Note {

}

fn stale_notes(note_path: &str) -> std::result::Result<Vec<String>, tantivy::TantivyError> {
    // For now we just reindex the entire directory on every invocation
    // But eventually, we will only reindex files with updated mtimes
    let note_dir = fs::read_dir(note_path).unwrap();

    let note_paths = note_dir
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .metadata()
                .unwrap()
                .file_type()
                .is_file()
        })
        .filter_map(|entry| entry.unwrap().path().to_str().map(|s| String::from(s)))
        .collect();

    return Ok(note_paths);
} 
    
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

fn main() -> anyhow::Result<()> {
    let query = parse_args();
    let config = config::parse("./config.toml")?;
    let index = index::Index::new(&config.index_dir);
    return Ok(());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
// #[macro_use]

use log::{debug, info};
use std::fs::File;
use std::path::{PathBuf, Path};
//use std::ffi::OsStr;

use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};

use std::process::Command; 

use clap::Arg;
use clap::{crate_description, crate_authors, crate_version, crate_name};

mod index;
mod repo;
mod config;
mod heap;

use repo::*;
use index::*;
use heap::Heap;

fn arg_parser<'a,'b>() -> clap::App<'a,'b> {
    clap::app_from_crate!()
        .arg(
            Arg::with_name("HEAP")
                .help("Path to the card heap")
                .short("h")
                .long("heap")
                .required(false)
                .takes_value(true)
        )
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
                .help("note path")))
        .subcommand(clap::SubCommand::with_name("init")
            .about("create a new notebook at PATH")
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

fn heap_path(args: &clap::ArgMatches) -> Result<PathBuf> {
    let path = if let Some(dir) = args.value_of("HEAP") {
        Ok(PathBuf::from(dir))
    } else {
        std::env::var("NB")
            .map(|s| PathBuf::from(s))
            .or(std::env::current_dir())
            .context("failed to find heap path")
    };

    path.and_then(|p|p.canonicalize().context("failed to canonicalize path"))
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app = arg_parser();

    //app.sync()?;
    let args = app.clone().get_matches();

    match args.subcommand() {
        ("init", Some(subargs)) => { Heap::init(subargs.value_of("PATH").unwrap())?; }
        ("edit", Some(subargs)) => { new_note(subargs.value_of("PATH").unwrap()) }
        ("search", Some(subargs)) => {
            let heap_path = heap_path(&args)?;
            debug!("heap_path: {:?}", heap_path);
            let mut heap = Heap::open(heap_path)?;
            heap.sync()?;
            let query = subargs.value_of("QUERYSTRING").unwrap();
            debug!("query: {:?}", query);
            heap.find(query)?;
        }
        _ => {
            app.clone().print_help()?;
        }
    }

    //app.state.save()?;

    return Ok(());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
use log::{debug};
use std::path::{PathBuf};

use anyhow::{Context, Result};

use clap::Arg;
use clap::{crate_description, crate_authors, crate_version, crate_name};

mod index;
mod repo;
//mod config;
mod heap;
mod card;

//use repo::*;
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
            .arg(Arg::with_name("PATH")
                .takes_value(true)
                .index(1)
                .required(false)
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
    let matches = app.clone().get_matches();

    if let Some(args) = matches.subcommand_matches("init") {
        Heap::init(args.value_of("PATH").unwrap())?;
        return Ok(())
    }

    match (matches.subcommand(), heap_path(&matches)) {
        (("add", Some(subargs)), Ok(heap_path)) => { 
            //let path = subargs.value_of("PATH").unwrap();
            Heap::open(heap_path)?.add_card(subargs.value_of("PATH"))?;
        }
        (("edit", Some(subargs)), Ok(heap_path)) => { 
            let path = subargs.value_of("PATH").unwrap();
            Heap::open(heap_path)?.edit_card(path)?;
        }
        (("search", Some(subargs)), Ok(heap_path)) => {
            let query = subargs.value_of("QUERYSTRING").unwrap();
            debug!("query: {:?}", query);
            let mut heap = Heap::open(heap_path)?;
            heap.sync()?;
            heap.find(query)?;
        }
        _ => {
            app.clone().print_help()?;
        }
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
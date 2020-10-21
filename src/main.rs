// #[macro_use]
// extern crate tantivy;

use log::{debug, error, info, warn};
use std::fs;
use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result};
use serde::Deserialize;

use derive_new::new;
use clap::{App, Arg};

use pulldown_cmark::{Event, Options, Parser, Tag};

mod git {
    use std::process::Command;

    pub struct GitNoteRepo {
        git_dir: String,
        indexed_commit: String, // Last indexed commit SHA1
    }

    impl GitNoteRepo {
        pub fn list_changes(&self) -> anyhow::Result<&str> {
            let output = Command::new("git").arg("status").output()?;
            return Ok("win");
        }
    }
}


mod config {
    //use toml::Value;
    use serde::Deserialize;
    use anyhow::{Context, Result};
    use std::fs::File;
    use std::io::Read;
    use log::{warn};
    
    #[derive(Deserialize)]
    pub struct Config {
        index_dir: String,
        notes_dir: String,
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
}

mod index {
    use anyhow::{Context, Result};
    use tantivy::collector::TopDocs;
    use tantivy::directory::MmapDirectory;
    use tantivy::query::QueryParser;
    use tantivy::schema::*;
    use tantivy::{Score, DocAddress};
    use tantivy::ReloadPolicy;
    use tantivy::SnippetGenerator;

    pub struct Index {
        reader: tantivy::IndexReader,
        writer: tantivy::IndexWriter,
        schema: tantivy::schema::Schema,
        queryparser: tantivy::query::QueryParser,
    }

    pub struct Note {
        path: String,
        fields: Vec<FieldValue>,

    }
    
    impl Index {
        pub fn new(index_dir: &str) -> anyhow::Result<Index> {

            let index_dir = MmapDirectory::open(index_dir)?;
            let schema = Index::build_schema()?;
            let index = tantivy::Index::open_or_create(index_dir, schema.clone())?;

            let mut writer = index.writer(50_000_000)?;

            let reader = index
                .reader_builder()
                .reload_policy(ReloadPolicy::OnCommit)
                .try_into()?;

            let queryparser = QueryParser::new(
                schema,
                vec![],
                tantivy::tokenizer::TokenizerManager::default());

            return Ok(Index {
                reader: reader,
                writer: writer,
                schema: schema,
                queryparser: queryparser,
            });
        }

        pub fn query(&self, query: &str) -> anyhow::Result<Vec<(Score, DocAddress)>> {

            let searcher = self.reader.searcher();
            let query = self.queryparser.parse_query(query)?;


            let title = self.schema.get_field("title")
                .context("failed to find 'title' in schema")?;
            let body = self.schema.get_field("body")
                .context("failed to find 'body' in schema")?;

            let snippet_generator = SnippetGenerator::create(&searcher, &*query, body)?;

            let top_docs: Vec<(Score,DocAddress)> = searcher.search(&query, &TopDocs::with_limit(10))?;

            for (score, doc_address) in top_docs {
                let doc = searcher.doc(doc_address)?;
                let snippet = snippet_generator.snippet_from_doc(&doc);
                println!("Document score {}:", score);
                println!("title: {}", doc.get_first(title).unwrap().text().unwrap());
                println!("snippet: {}", snippet.fragments());
            }

            return Ok(vec![]);
        }
    
        fn build_schema() -> anyhow::Result<tantivy::schema::Schema> {
            let mut schema_builder = Schema::builder();
    
            schema_builder.add_text_field("path", STRING | STORED);
            schema_builder.add_text_field("title", TEXT | STORED);
            schema_builder.add_text_field("body", TEXT | STORED);
            schema_builder.add_text_field("mtime", TEXT);
            schema_builder.add_text_field("section", TEXT | STORED);
    
            let schema = schema_builder.build();
    
            return Ok(schema);
        }

        pub fn delete(&mut self, note: &Note) -> anyhow::Result<()> {

            let path_field = &self.schema.get_field("path").unwrap();
            self.writer.delete_term(path_field, note.path);
            self.writer.commit()?;
            return Ok(());
        }
    
        pub fn add(&mut self, note: Note) -> anyhow::Result<()> {

            self.delete(&note);

            let path_field = self.schema.get_field("path").unwrap();
    
            let mut doc = Document::default();
            doc.add_text(path_field, note.path);

            for field in note.fields {
                doc.add(field);
            }
   
            self.writer.add_document(doc);
            self.writer.commit()?;
            return Ok(());
        }

        pub fn run(&mut self, ops: Vec<tantivy::UserOperation>) -> anyhow::Result<()> {
            self.writer.run(ops);
            self.writer.commit()?;
            return Ok(());
        }
    }

    
}

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
    
//fn classify_markdown () -> () {
//    let mut firstheading = true;
//
//    let mut inheading = false;
//    for event in pulldown_cmark::Parser::new(note.body) {
//        match event {
//            Event::Start(Tag::Heading(_)) => {
//                inheading = true;
//            }
//            Event::End(Tag::Heading(_)) => {
//                inheading = false;
//            }
//            Event::Text(text) => {
//                if inheading {
//                    if firstheading {
//                        firstheading = false;
//                        doc.add_text(title_field, &text);
//                    } else {
//                        doc.add_text(section_field, &text);
//                    }
//                }
//            }
//            _ => (),
//        }
//    }
//}

use git::*;
use config::*;
use index::*;

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
    return Ok(());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
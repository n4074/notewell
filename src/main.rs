#[macro_use]
extern crate tantivy;

use std::fs;
use std::error;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::directory::MmapDirectory;
use tantivy::ReloadPolicy;
use tantivy::SnippetGenerator;

use pulldown_cmark::{Parser, Options, Event, Tag};


struct Config<'a> {
    index_dir: &'a str,
    notes_dir: &'a str 
}

const config: Config = Config { 
    index_dir: "/tmp/index", 
    notes_dir: "/tmp/notes" 
};

fn build_schema() -> std::result::Result<tantivy::schema::Schema, tantivy::TantivyError> {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("path", STRING | STORED);
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    schema_builder.add_text_field("mtime", TEXT);
    schema_builder.add_text_field("section", TEXT | STORED);

    let schema = schema_builder.build();

    return Ok(schema);
}


fn index_note(note_path: &str, schema: &tantivy::schema::Schema, index_writer: &mut tantivy::IndexWriter) -> std::result::Result<(), tantivy::TantivyError> {

    let path_field = schema.get_field("path").unwrap();
    let title_field = schema.get_field("title").unwrap();
    let body_field = schema.get_field("body").unwrap();
    let section_field = schema.get_field("section").unwrap();

    let existing = Term::from_field_text(path_field, note_path);
    index_writer.delete_term(existing);

    let mut doc = Document::default();
    let body = fs::read_to_string(note_path)?;

    doc.add_text(path_field, &note_path);
    doc.add_text(body_field, &body);

    let mut firstheading = true;
    let mut inheading = false;
    for event in pulldown_cmark::Parser::new(&body) {
        match event {
            Event::Start(Tag::Heading(_)) => {
                inheading = true;
            },
            Event::End(Tag::Heading(_)) => {
                inheading = false;
                
            },
            Event::Text(text) => { 
                if inheading { 
                    if firstheading {
                        firstheading = false;
                        doc.add_text(title_field, &text);
                    } else {
                        doc.add_text(section_field, &text); 
                    }
                }
            },
            _ => ()
        }
    }

    index_writer.add_document(doc);
    index_writer.commit()?;

    return Ok(());
}

fn main() -> std::result::Result<(), tantivy::TantivyError> {
    //let index_path = TempDir::new("tantivy_example_dir")?;

    let schema = build_schema()?;

    let index_dir = MmapDirectory::open(config.index_dir)?;

    let index = Index::open_or_create(index_dir, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?;

    index_note("/tmp/notes/test.md", &schema, &mut index_writer)?;

    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();
    let section = schema.get_field("section").unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![title, body, section]);

    let query = query_parser.parse_query("wat")?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    let snippet_generator = SnippetGenerator::create(&searcher, &*query, body)?;

    for (score, doc_address) in top_docs {
        let doc = searcher.doc(doc_address)?;
        let snippet = snippet_generator.snippet_from_doc(&doc);
        println!("Document score {}:", score);
        println!("title: {}", doc.get_first(title).unwrap().text().unwrap());
        println!("snippet: {}", snippet.to_html());
        println!("snippet nh: {}", snippet.fragments());
    }

    return Ok(());
}

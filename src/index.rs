use anyhow::Context;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Score, DocAddress};
use tantivy::ReloadPolicy;
use tantivy::SnippetGenerator;
use tantivy::UserOperation;
//use tantivy::schema::Field;
//use anyhow::anyhow;

use std::path::{PathBuf,Path};

pub struct Index {
    _index: tantivy::Index,
    reader: tantivy::IndexReader,
    writer: tantivy::IndexWriter,
    schema: tantivy::schema::Schema,
    queryparser: tantivy::query::QueryParser,
    transactions: Vec<tantivy::UserOperation>,
}

pub struct Note {
    schema: tantivy::schema::Schema,
    pub path: PathBuf,
    doc: tantivy::Document,
}

const DEFAULT_FIELD_NAME: &str = "body";

impl Note {

    fn new(schema: tantivy::schema::Schema, path: &Path) -> Note {
        let mut doc = tantivy::Document::default();
        let pathfield = schema.get_field("path").unwrap();
        doc.add_text(pathfield, path.to_str().unwrap());
        Note {
            schema,
            path: path.to_owned(),
            doc
        }
    }

    fn add_field(&mut self, name: &str, content: &str) -> &Note {
        let field = self.schema.get_field(name).unwrap();
        self.doc.add_text(field, &content);
        self
    }

    pub fn body(&mut self, content: &str) -> &Note {
        println!("here: {:?}", content);
        self.add_field("body", content)
    }

    pub fn document(self) -> Document {
        self.doc
    }
}

impl Index {
    pub fn open_or_create<P: AsRef<Path>>(dir: P) -> anyhow::Result<Index> {

        std::fs::create_dir_all(&dir)?;
        let dir = MmapDirectory::open(&dir)?;
        let schema = Self::build_schema()?;
        let index = tantivy::Index::open_or_create(dir, schema.clone())?;
        let writer = index.writer(50_000_000)?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let queryparser = QueryParser::new(
            schema.clone(),
            vec![schema.get_field(DEFAULT_FIELD_NAME).unwrap()],
            tantivy::tokenizer::TokenizerManager::default());


        let transactions = vec!();

        return Ok(Index {
            _index: index,
            reader: reader,
            writer: writer,
            schema: schema,
            queryparser: queryparser,
            transactions
        });
    }

    pub fn reload(&self) -> anyhow::Result<()> {
        self.reader.reload().context("Failed to reload index")
    }

    pub fn query(&self, query: &str) -> anyhow::Result<Vec<Document>> {
        let searcher = self.reader.searcher();

        let query = self.queryparser.parse_query(query)?;

        let body = self.schema.get_field("body")
            .context("failed to find 'body' in schema")?;
        let path = self.schema.get_field("path")
            .context("failed to find 'path' in schema")?;

        let snippet_generator = SnippetGenerator::create(&searcher, &*query, body)?;

        let top_docs: Vec<(Score,DocAddress)> = searcher.search(&query, &TopDocs::with_limit(10))?;

        //for (score, doc_address) in top_docs {
        //    let doc = searcher.doc(doc_address)?;
        //    let snippet = snippet_generator.snippet_from_doc(&doc);
        //    println!("{}", self.schema.to_json(&doc));
        //    println!("Document score {}:", score);
        //    println!("path: {}", doc.get_first(path).unwrap().text().unwrap());
        //    println!("snippet: {}", snippet.fragments());
        //}

        let docs: Vec<Document> = top_docs.iter().map(|(_,addr)| searcher.doc(*addr).unwrap()).collect();

        return Ok(docs);
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

    pub fn notebuilder(&self, path: &Path) -> Note {
        Note::new(self.schema.clone(), path)
    }

    pub fn add(&mut self, path: &Path, note: Note) {
        println!("Adding document {:?}", path);
        self.transactions.push(UserOperation::Add(note.document()));
    }

    pub fn delete(&mut self, path: &Path) {
        println!("Deleting document {:?}", path);
        let path_field = self.schema.get_field("path").unwrap();
        let term = Term::from_field_text(path_field, path.to_str().unwrap());
        self.transactions.push(UserOperation::Delete(term));
    }

    pub fn commit(&mut self) -> anyhow::Result<u64> {
        let transactions = self.transactions.drain(..).collect();
        self.writer.run(transactions);
        let res = self.writer.commit().context("Failed to commit")?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use tempfile::tempdir;
    use anyhow::Context;
    use std::io::Write;
    use git2::{Repository,Signature};

    #[test]
    fn index_git_repo() -> anyhow::Result<()> {
        Ok(())
    }
}
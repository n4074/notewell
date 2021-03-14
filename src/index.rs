use anyhow::{Context, Result};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Score, DocAddress};
use tantivy::ReloadPolicy;
use tantivy::SnippetGenerator;

use std::path::Path;

pub struct Index {
    index: tantivy::Index,
    reader: tantivy::IndexReader,
    writer: tantivy::IndexWriter,
    pub schema: tantivy::schema::Schema,
    queryparser: tantivy::query::QueryParser,
}

impl Index {
    pub fn open_or_create(dir: &Path) -> anyhow::Result<Index> {

        std::fs::create_dir_all(dir)?;
        let dir = MmapDirectory::open(dir)?;
        let schema = Self::build_schema()?;
        let index = tantivy::Index::open_or_create(dir, schema.clone())?;
        let writer = index.writer(50_000_000)?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let queryparser = QueryParser::new(
            schema.clone(),
            vec![],
            tantivy::tokenizer::TokenizerManager::default());

        return Ok(Index {
            index: index,
            reader: reader,
            writer: writer,
            schema: schema,
            queryparser: queryparser,
        });
    }

    pub fn reload(&self) -> anyhow::Result<()> {
        self.reader.reload().context("Failed to reload index")
    }

    pub fn query(&self, query: &str) -> anyhow::Result<Vec<(Score, DocAddress)>> {
        let searcher = self.reader.searcher();
        let query = self.queryparser.parse_query(query)?;

        let title = self.schema.get_field("title")
            .context("failed to find 'title' in schema")?;
        let body = self.schema.get_field("body")
            .context("failed to find 'body' in schema")?;
        let path = self.schema.get_field("path")
            .context("failed to find 'path' in schema")?;

        let snippet_generator = SnippetGenerator::create(&searcher, &*query, body)?;

        let top_docs: Vec<(Score,DocAddress)> = searcher.search(&query, &TopDocs::with_limit(10))?;

        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let snippet = snippet_generator.snippet_from_doc(&doc);
            println!("Document score {}:", score);
            println!("path: {}", doc.get_first(path).unwrap().text().unwrap());
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

    pub fn delete(&mut self, path: &Path) -> anyhow::Result<()> {
        let path_field = self.schema.get_field("path").unwrap();
        let existing = Term::from_field_text(path_field, path.to_str().unwrap());
        self.writer.delete_term(existing);
        self.writer.commit()?;
        return Ok(());
    }

    pub fn add(&mut self, path: &Path, fields: Vec<FieldValue>) -> anyhow::Result<()> {

        self.delete(path)?;

        let path_field = self.schema.get_field("path").unwrap();

        let mut doc = Document::default();
        doc.add_text(path_field, path.to_str().unwrap());

        for field in fields {
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

#[cfg(test)]
mod tests {
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
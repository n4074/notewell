use anyhow::Result;
use tantivy::Document;

use crate::index::QueryResult;

pub fn list_results(docs: Vec<QueryResult>) -> Result<()> {
    for doc in docs {
        println!("result: {:?}", doc);
    }

    Ok(())
}
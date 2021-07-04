use tantivy::Document;

fn list_results(docs: Vec<Document>>) {
    for doc in docs {
        println!("path: {}", doc.get_first(path).unwrap().text().unwrap());
    }
}
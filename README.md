<div align="center">
  <h1><a href="https://crates.io/crates/chromadb">ChromaDB-rs</a></h1>
  <h3>A Rust client library for the Chroma vector database.</h3>
  <a href="https://crates.io/crates/chromadb"><img src="https://img.shields.io/crates/v/chromadb.svg" alt="Crates.io"></a>
  <a href="https://github.com/Anush008/chromadb-rs/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-mit-blue.svg" alt="MIT Licensed"></a>
  <a href="https://github.com/Anush008/chromadb-rs/actions/workflows/cargo-test.yml"><img src="https://github.com/Anush008/chromadb-rs/actions/workflows/release.yml/badge.svg?branch=master" alt="Tests"></a>
</div>

## 💾 Installing the library

```shell
cargo add chromadb
```

## 📖 Documentation

The library reference can be found [here](https://docs.rs/chromadb).

## 🔍 Overview

#### The library provides 2 modules to interact with the ChromaDB server via API V1

* `client` - To interface with the ChromaDB server.
* `collection` - To interface with an associated ChromaDB collection.

#### You can connect to ChromaDB by instantiating a [ChromaClient](https://docs.rs/chromadb/latest/chromadb/v1/client/struct.ChromaClient.html)

 ```rust
use chromadb::client::{ChromaAuthMethod, ChromaClient, ChromaClientOptions, ChromaTokenHeader};
use chromadb::collection::{ChromaCollection, GetOptions, GetResult, CollectionEntries};

// With default ChromaClientOptions
// Defaults to http://localhost:8000
let client: ChromaClient = ChromaClient::new(Default::default());

// With custom ChromaClientOptions
let auth = ChromaAuthMethod::TokenAuth {
    token: "<TOKEN>".to_string(),
    header: ChromaTokenHeader::Authorization
};
let client: ChromaClient = ChromaClient::new(ChromaClientOptions {
    url: Some("<CHROMADB_URL>".into()),
    database: "<DATABASE>".into(),
    auth
});
```

#### Now that a client is instantiated, we can interface with the ChromaDB server

 ```rust
use serde_json::json;

// Get or create a collection with the given name and no metadata.
let collection: ChromaCollection = client.get_or_create_collection("my_collection", None).await?;

// Get the UUID of the collection
let collection_uuid = collection.id();
println!("Collection UUID: {}", collection_uuid);
```

### With a collection instance, we can perform queries on the database

```rust
// Upsert some embeddings with documents and no metadata.
let collection_entries = CollectionEntries {
    ids: vec!["demo-id-1".into(), "demo-id-2".into()],
    embeddings: Some(vec![vec![0.0_f32; 768], vec![0.0_f32; 768]]),
    metadatas: None,
    documents: Some(vec![
        "Some document about 9 octopus recipies".into(),
        "Some other document about DCEU Superman Vs CW Superman".into()
    ])
 };
 
let result: bool = collection.upsert(collection_entries, None).await?;

// Create a filter object to filter by document content.
let where_document = json!({
    "$contains": "Superman"
});
 
// Get embeddings from a collection with filters and limit set to 1. 
// An empty IDs vec will return all embeddings.
let get_query = GetOptions {
     ids: vec![],
     where_metadata: None,
     limit: Some(1),
     offset: None,
     where_document: Some(where_document),
     include: Some(vec!["documents".into(),"embeddings".into()])
 };
let get_result: GetResult = collection.get(get_query).await?;
println!("Get result: {:?}", get_result);

```

Find more information about the available filters and options in the [get()](https://docs.rs/chromadb/latest/chromadb/v1/collection/struct.ChromaCollection.html#method.get) documentation.

### Performing a similarity search

```rust
//Instantiate QueryOptions to perform a similarity search on the collection
//Alternatively, an embedding_function can also be provided with query_texts to perform the search
let query = QueryOptions {
    query_texts: None,
    query_embeddings: Some(vec![vec![0.0_f32; 768], vec![0.0_f32; 768]]),
    where_metadata: None,
    where_document: None,
    n_results: Some(5),
    include: None,
 };
 
let query_result: QueryResult = collection.query(query, None).await?;
println!("Query result: {:?}", query_result);
```

### Support for Embedding providers

This crate has built-in support for OpenAI and SBERT embeddings.

To use [OpenAI](https://platform.openai.com/docs/guides/embeddings) embeddings, enable the `openai` feature in your Cargo.toml.

 ```rust
let collection: ChromaCollection = client.get_or_create_collection("openai_collection", None).await?;

let collection_entries = CollectionEntries {
   ids: vec!["demo-id-1", "demo-id-2"],
   embeddings: None,
   metadatas: None,
   documents: Some(vec![
            "Some document about 9 octopus recipies",
            "Some other document about DCEU Superman Vs CW Superman"])
 };

// Use OpenAI embeddings
let openai_embeddings = OpenAIEmbeddings::new(Default::default());

collection.upsert(collection_entries, Some(Box::new(openai_embeddings))).await?;
 ```

## Sponsors

[![OpenSauced logo](https://raw.githubusercontent.com/open-sauced/assets/main/logos/logo-on-dark.png)](https://opensauced.pizza?utm_source=chromadbrs&utm_medium=github&utm_campaign=sponsorship)

[OpenSauced](https://opensauced.pizza?utm_source=chromadbrs&utm_medium=github&utm_campaign=sponsorship) provides insights into open source projects by using data science in git commits.

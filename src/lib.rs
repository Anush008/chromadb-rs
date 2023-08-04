//! [ChromaDB](https://www.trychroma.com/) client library for Rust.
//!
//! The library provides 2 modules to interact with the ChromaDB server via API V1:
//! * `client` - To interface with the ChromaDB server.
//! * `collection` - To interface with an associated ChromaDB collection.
//!
//! ### Instantiating [ChromaClient](crate::v1::ChromaClient)
//! ```
//! use chromadb::v1::client::{ChromaClient, ChromaClientOptions};
//! use chromadb::v1::collection::{ChromaCollection, GetResult, GetOptions};
//! use serde_json::json;
//!
//!# async fn doc_client_demo() -> anyhow::Result<()> {
//! // With default ChromaClientOptions
//! // Defaults to http://localhost:8000
//! let client: ChromaClient = ChromaClient::new(Default::default());
//!
//! // With custom ChromaClientOptions
//! let client: ChromaClient = ChromaClient::new(ChromaClientOptions { url: "<CHROMADB_URL>".into() });
//!
//! # Ok(())
//! # }
//! ```
//! Now that a client is instantiated, we can interface with the ChromaDB server and execute queries.
//!
//! ### Collection Queries
//!
//! ```
//!# use chromadb::v1::ChromaClient;
//!# use chromadb::v1::collection::{ChromaCollection, GetResult, CollectionEntries, GetOptions};
//!# use serde_json::json;
//!# async fn doc_client_create_collection(client: &ChromaClient) -> anyhow::Result<()> {
//! // Get or create a collection with the given name and no metadata.
//! let collection: ChromaCollection = client.get_or_create_collection("my_collection", None).await?;
//!
//! // Get the UUID of the collection
//! let collection_uuid = collection.id();
//! println!("Collection UUID: {}", collection_uuid);
//!
//! // Upsert some embeddings with documents and no metadata.
//! let collection_entries = CollectionEntries {
//!    ids: vec!["demo-id-1".into(), "demo-id-2".into()],
//!    embeddings: Some(vec![vec![0.0_f64; 768], vec![0.0_f64; 768]]),
//!    metadatas: None,
//!    documents: Some(vec![
//!        "Some document about 9 octopus recipies".into(),
//!        "Some other document about DCEU Superman Vs CW Superman".into()
//!    ])
//! };
//!
//! let result: bool = collection.upsert(collection_entries, None).await?;
//!
//! // Create a filter object to filter by document content.
//! let where_document = json!({
//!    "$contains": "Superman"
//!     });
//!
//! // Get embeddings from a collection with filters and limit set to 1.
//! // An empty IDs vec will return all embeddings.
//!
//! let get_query = GetOptions {
//!     ids: vec![],
//!     where_metadata: None,
//!     limit: Some(1),
//!     offset: None,
//!     where_document: Some(where_document),
//!     include: Some(vec!["documents".into(),"embeddings".into()])
//! };
//!
//! let get_result: GetResult = collection.get(get_query).await?;
//! println!("Get result: {:?}", get_result);
//!# Ok(())
//!# }
//! ```
//!Find more information about on the available filters and options in the [get()](crate::v1::ChromaCollection::get) documentation.
//! 
//! 
//! ### Perform a similarity search.
//! ```
//!# use chromadb::v1::collection::{ChromaCollection, QueryResult, QueryOptions};
//!# use serde_json::json;
//!# async fn doc_query_collection(collection: &ChromaCollection) -> anyhow::Result<()> {
//! //Instantiate QueryOptions to perform a similarity search on the collection
//! //Alternatively, an embedding_function can also be provided with query_texts to perform the search
//! let query = QueryOptions {
//!     query_texts: None,
//!     query_embeddings: Some(vec![vec![0.0_f64; 768], vec![0.0_f64; 768]]),
//!     where_metadata: None,
//!     where_document: None,
//!     n_results: Some(5),
//!     include: None,
//! };
//! 
//! let query_result: QueryResult = collection.query(query, None).await?;
//! println!("Query result: {:?}", query_result);
//!# Ok(()) 
//!# }
//! ```
//! 
pub mod v1;

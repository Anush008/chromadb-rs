use anyhow::bail;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashSet, sync::Arc, vec};

use super::{
    api::APIClientAsync,
    commons::{Documents, Embedding, Embeddings, Metadata, Metadatas, Result},
    embeddings::EmbeddingFunction,
};

/// A collection representation for interacting with the associated ChromaDB collection.
#[derive(Deserialize, Debug)]
pub struct ChromaCollection {
    #[serde(skip)]
    pub(super) api: Arc<APIClientAsync>,
    pub(super) id: String,
    pub(super) metadata: Option<Metadata>,
    pub(super) name: String,
}

impl ChromaCollection {
    /// Get the UUID of the collection.
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    /// Get the name of the collection.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the metadata of the collection.
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// The total number of embeddings added to the database.
    pub async fn count(&self) -> Result<usize> {
        let path = format!("/collections/{}/count", self.id);
        let response = self.api.get(&path).await?;
        let count = response.json::<usize>().await?;
        Ok(count)
    }

    /// Modify the name/metadata of a collection.
    ///
    /// # Arguments
    ///
    /// * `name` - The new name of the collection. Must be unique.
    /// * `metadata` - The new metadata of the collection. Must be a JSON object with keys and values that are either numbers, strings or floats.
    ///
    /// # Errors
    ///
    /// * If the collection name is invalid
    pub async fn modify(&self, name: Option<&str>, metadata: Option<&Metadata>) -> Result<()> {
        let json_body = json!({
            "new_name": name,
            "new_metadata": metadata,
        });
        let path = format!("/collections/{}", self.id);
        self.api.put(&path, Some(json_body)).await?;
        Ok(())
    }

    /// Add embeddings to the data store. Ignore the insert if the ID already exists.
    ///
    /// # Arguments
    ///
    /// * `ids` - The ids to associate with the embeddings.
    /// * `embeddings` -  The embeddings to add. If None, embeddings will be computed based on the documents using the provided embedding_function. Optional.
    /// * `metadata` - The metadata to associate with the embeddings. When querying, you can filter on this metadata. Optional.
    /// * `documents` - The documents to associate with the embeddings. Optional.
    /// * `embedding_function` - The function to use to compute the embeddings. If None, embeddings must be provided. Optional.
    ///
    /// # Errors
    ///
    /// * If you don't provide either embeddings or documents
    /// * If the length of ids, embeddings, metadatas, or documents don't match
    /// * If you provide duplicates in ids, empty ids
    /// * If you provide documents and don't provide an embedding function when embeddings is None
    /// * If you provide an embedding function and don't provide documents
    /// * If you provide both embeddings and embedding_function
    ///
    pub async fn add<'a>(
        &self,
        collection_entries: CollectionEntries<'a>,
        embedding_function: Option<Box<dyn EmbeddingFunction>>,
    ) -> Result<Value> {
        let collection_entries = validate(true, collection_entries, embedding_function)?;

        let CollectionEntries {
            ids,
            embeddings,
            metadatas,
            documents,
        } = collection_entries;

        let json_body = json!({
            "ids": ids,
            "embeddings": embeddings,
            "metadatas": metadatas,
            "documents": documents,
        });

        let path = format!("/collections/{}/add", self.id);
        let response = self.api.post(&path, Some(json_body)).await?;
        let response = response.json::<Value>().await?;

        Ok(response)
    }

    /// Add embeddings to the data store. Update the entry if an ID already exists.
    ///
    /// # Arguments
    ///
    /// * `ids` - The ids to associate with the embeddings.
    /// * `embeddings` -  The embeddings to add. If None, embeddings will be computed based on the documents using the provided embedding_function. Optional.
    /// * `metadata` - The metadata to associate with the embeddings. When querying, you can filter on this metadata. Optional.
    /// * `documents` - The documents to associate with the embeddings. Optional.
    /// * `embedding_function` - The function to use to compute the embeddings. If None, embeddings must be provided. Optional.
    ///
    /// # Errors
    ///
    /// * If you don't provide either embeddings or documents
    /// * If the length of ids, embeddings, metadatas, or documents don't match
    /// * If you provide duplicates in ids, empty ids
    /// * If you provide documents and don't provide an embedding function when embeddings is None
    /// * If you provide an embedding function and don't provide documents
    /// * If you provide both embeddings and embedding_function
    ///
    pub async fn upsert<'a>(
        &self,
        collection_entries: CollectionEntries<'a>,
        embedding_function: Option<Box<dyn EmbeddingFunction>>,
    ) -> Result<Value> {
        let collection_entries = validate(true, collection_entries, embedding_function)?;

        let CollectionEntries {
            ids,
            embeddings,
            metadatas,
            documents,
        } = collection_entries;

        let json_body = json!({
            "ids": ids,
            "embeddings": embeddings,
            "metadatas": metadatas,
            "documents": documents,
        });

        let path = format!("/collections/{}/upsert", self.id);
        let response = self.api.post(&path, Some(json_body)).await?;
        let response = response.json::<Value>().await?;

        Ok(response)
    }

    /// Get embeddings and their associated data from the collection. If no ids or filter is provided returns all embeddings up to limit starting at offset.
    ///
    /// # Arguments
    ///
    /// * `ids` - The ids of the embeddings to get. Optional..
    /// * `where_metadata` - Used to filter results by metadata. E.g. `{ "$and": [{"foo": "bar"}, {"price": {"$gte": 4.20}}] }`. See <https://docs.trychroma.com/usage-guide#filtering-by-metadata> for more information on metadata filters. Optional.
    /// * `limit` - The maximum number of documents to return. Optional.
    /// * `offset` - The offset to start returning results from. Useful for paging results with limit. Optional.
    /// * `where_document` - Used to filter by the documents. E.g. {"$contains": "hello"}. See <https://docs.trychroma.com/usage-guide#filtering-by-document-contents> for more information on document content filters. Optional.
    /// * `include` - A list of what to include in the results. Can contain `"embeddings"`, `"metadatas"`, `"documents"`. Ids are always included. Defaults to `["metadatas", "documents"]`. Optional.
    ///
    pub async fn get(&self, get_options: GetOptions) -> Result<GetResult> {
        let GetOptions {
            ids,
            where_metadata,
            limit,
            offset,
            where_document,
            include,
        } = get_options;
        let mut json_body = json!({
            "ids": ids,
            "where": where_metadata,
            "limit": limit,
            "offset": offset,
            "where_document": where_document,
            "include": include
        });

        json_body
            .as_object_mut()
            .unwrap()
            .retain(|_, v| !v.is_null());

        let path = format!("/collections/{}/get", self.id);
        let response = self.api.post(&path, Some(json_body)).await?;
        let get_result = response.json::<GetResult>().await?;
        Ok(get_result)
    }

    /// Update the embeddings, metadatas or documents for provided ids.
    ///
    /// # Arguments
    ///
    /// * `ids` - The ids to associate with the embeddings.
    /// * `embeddings` -  The embeddings to add. If None, embeddings will be computed based on the documents using the provided embedding_function. Optional.
    /// * `metadata` - The metadata to associate with the embeddings. When querying, you can filter on this metadata. Optional.
    /// * `documents` - The documents to associate with the embeddings. Optional.
    /// * `embedding_function` - The function to use to compute the embeddings. If None, embeddings must be provided. Optional.
    ///
    /// # Errors
    ///
    /// * If the length of ids, embeddings, metadatas, or documents don't match
    /// * If you provide duplicates in ids, empty ids
    /// * If you provide documents and don't provide an embedding function when embeddings is None
    /// * If you provide an embedding function and don't provide documents
    /// * If you provide both embeddings and embedding_function
    ///
    pub async fn update<'a>(
        &self,
        collection_entries: CollectionEntries<'a>,
        embedding_function: Option<Box<dyn EmbeddingFunction>>,
    ) -> Result<bool> {
        let collection_entries = validate(false, collection_entries, embedding_function)?;

        let CollectionEntries {
            ids,
            embeddings,
            metadatas,
            documents,
        } = collection_entries;

        let json_body = json!({
            "ids": ids,
            "embeddings": embeddings,
            "metadatas": metadatas,
            "documents": documents,
        });

        let path = format!("/collections/{}/update", self.id);
        let response = self.api.post(&path, Some(json_body)).await?;
        let response = response.json::<bool>().await?;

        Ok(response)
    }

    ///Get the n_results nearest neighbor embeddings for provided query_embeddings or query_texts.
    ///
    /// # Arguments
    ///
    /// * `query_embeddings` - The embeddings to get the closest neighbors of. Optional.
    /// * `query_texts` -  The document texts to get the closest neighbors of. Optional.
    /// * `n_results` - The number of neighbors to return for each query_embedding or query_texts. Optional.
    /// * `where_metadata` - Used to filter results by metadata. E.g. {"$and": ["color" : "red", "price": {"$gte": 4.20}]}. Optional.
    /// * `where_document` - Used to filter results by documents. E.g. {$contains: "some text"}. Optional.
    /// * `include` - A list of what to include in the results. Can contain "embeddings", "metadatas", "documents", "distances". Ids are always included. Defaults to ["metadatas", "documents", "distances"]. Optional.
    /// * `embedding_function` - The function to use to compute the embeddings. If None, embeddings must be provided. Optional.
    ///
    /// # Errors
    ///
    /// * If you don't provide either query_embeddings or query_texts
    /// * If you provide both query_embeddings and query_texts
    /// * If you provide query_texts and don't provide an embedding function when embeddings is None
    ///
    pub async fn query<'a>(
        &self,
        query_options: QueryOptions<'a>,
        embedding_function: Option<Box<dyn EmbeddingFunction>>,
    ) -> Result<QueryResult> {
        let QueryOptions {
            mut query_embeddings,
            query_texts,
            n_results,
            where_metadata,
            where_document,
            include,
        } = query_options;
        if query_embeddings.is_some() && query_texts.is_some() {
            bail!("You can only provide query_embeddings or query_texts, not both");
        } else if query_embeddings.is_none() && query_texts.is_none() {
            bail!("You must provide either query_embeddings or query_texts");
        } else if query_texts.is_some() && embedding_function.is_none() {
            bail!("You must provide an embedding function when providing query_texts");
        } else if query_embeddings.is_none() && embedding_function.is_some() {
            query_embeddings = Some(
                embedding_function
                    .unwrap()
                    .embed(query_texts.as_ref().unwrap())?,
            );
        };

        let mut json_body = json!({
            "query_embeddings": query_embeddings,
            "n_results": n_results,
            "where": where_metadata,
            "where_document": where_document,
            "include": include
        });

        json_body
            .as_object_mut()
            .unwrap()
            .retain(|_, v| !v.is_null());

        let path = format!("/collections/{}/query", self.id);
        let response = self.api.post(&path, Some(json_body)).await?;
        let query_result = response.json::<QueryResult>().await?;
        Ok(query_result)
    }

    ///Get the first entries in the collection up to the limit
    ///
    /// # Arguments
    ///
    /// * `limit` - The number of entries to return.
    ///
    pub async fn peek(&self, limit: usize) -> Result<GetResult> {
        let get_query = GetOptions {
            ids: vec![],
            where_metadata: None,
            limit: Some(limit),
            offset: None,
            where_document: None,
            include: None,
        };
        self.get(get_query).await
    }

    /// Delete the embeddings based on ids and/or a where filter. Deletes all the entries if None are provided
    ///
    /// # Arguments
    ///
    /// * `ids` - The ids of the embeddings to delete. Optional
    /// * `where_metadata` -  Used to filter deletion by metadata. E.g. {"$and": ["color" : "red", "price": {"$gte": 4.20}]}. Optional.
    /// * `where_document` - Used to filter the deletion by the document content. E.g. {$contains: "some text"}. Optional.. Optional.
    ///
    pub async fn delete(
        &self,
        ids: Option<Vec<&str>>,
        where_metadata: Option<Value>,
        where_document: Option<Value>,
    ) -> Result<Vec<String>> {
        let json_body = json!({
            "ids": ids,
            "where": where_metadata,
            "where_document": where_document,
        });

        let path = format!("/collections/{}/delete", self.id);
        let response = self.api.post(&path, Some(json_body)).await?;
        let response = response.json::<Vec<String>>().await?;

        Ok(response)
    }
}

#[derive(Deserialize, Debug)]
pub struct GetResult {
    pub ids: Vec<String>,
    pub metadatas: Option<Vec<Option<Vec<Option<Metadata>>>>>,
    pub documents: Option<Vec<Option<String>>>,
    pub embeddings: Option<Vec<Option<Embedding>>>,
}

#[derive(Serialize, Debug, Default)]
pub struct GetOptions {
    pub ids: Vec<String>,
    pub where_metadata: Option<Value>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub where_document: Option<Value>,
    pub include: Option<Vec<String>>,
}

#[derive(Serialize, Debug, Default)]
pub struct QueryOptions<'a> {
    pub query_embeddings: Option<Embeddings>,
    pub query_texts: Option<Vec<&'a str>>,
    pub n_results: Option<usize>,
    pub where_metadata: Option<Value>,
    pub where_document: Option<Value>,
    pub include: Option<Vec<&'a str>>,
}

#[derive(Deserialize, Debug)]
pub struct QueryResult {
    pub ids: Vec<Vec<String>>,
    pub metadatas: Option<Vec<Option<Vec<Option<Metadata>>>>>,
    pub documents: Option<Vec<Option<Vec<Option<String>>>>>,
    pub embeddings: Option<Vec<Option<Vec<Embedding>>>>,
    pub distances: Option<Vec<Option<Vec<f32>>>>,
}

#[derive(Serialize, Debug, Default)]
pub struct CollectionEntries<'a> {
    pub ids: Vec<&'a str>,
    pub metadatas: Option<Metadatas>,
    pub documents: Option<Documents<'a>>,
    pub embeddings: Option<Embeddings>,
}

fn validate(
    require_embeddings_or_documents: bool,
    collection_entries: CollectionEntries,
    embedding_function: Option<Box<dyn EmbeddingFunction>>,
) -> Result<CollectionEntries> {
    let CollectionEntries {
        ids,
        mut embeddings,
        metadatas,
        documents,
    } = collection_entries;
    if require_embeddings_or_documents && embeddings.is_none() && documents.is_none() {
        bail!("Embeddings and documents cannot both be None",);
    }

    if embeddings.is_none() && documents.is_some() && embedding_function.is_none() {
        bail!(
            "embedding_function cannot be None if documents are provided and embeddings are None",
        );
    }

    if embeddings.is_some() && embedding_function.is_some() {
        bail!("embedding_function should be None if embeddings are provided",);
    }

    if embeddings.is_none() && documents.is_some() && embedding_function.is_some() {
        embeddings = Some(
            embedding_function
                .unwrap()
                .embed(documents.as_ref().unwrap())?,
        );
    }

    for id in &ids {
        if id.is_empty() {
            bail!("Found empty string in IDs");
        }
    }

    if (embeddings.is_some() && embeddings.as_ref().unwrap().len() != ids.len())
        || (metadatas.is_some() && metadatas.as_ref().unwrap().len() != ids.len())
        || (documents.is_some() && documents.as_ref().unwrap().len() != ids.len())
    {
        bail!("IDs, embeddings, metadatas, and documents must all be the same length",);
    }

    let unique_ids: HashSet<_> = ids.iter().collect();
    if unique_ids.len() != ids.len() {
        let duplicate_ids: Vec<_> = ids
            .iter()
            .filter(|id| ids.iter().filter(|x| x == id).count() > 1)
            .collect();
        bail!(
            "Expected IDs to be unique, found duplicates for: {:?}",
            duplicate_ids
        );
    }
    Ok(CollectionEntries {
        ids,
        metadatas,
        documents,
        embeddings,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::r#async::{
        collection::{CollectionEntries, GetOptions, QueryOptions},
        embeddings::MockEmbeddingProvider,
        ChromaClient,
    };

    const TEST_COLLECTION: &str = "21-recipies-for-octopus";

    #[tokio::test]
    async fn test_modify_collection() {
        let client = ChromaClient::new(Default::default());

        let collection = client
            .get_or_create_collection(TEST_COLLECTION, None)
            .await
            .unwrap();

        //Test for setting invalid collection name. Should fail.
        assert!(collection
            .modify(Some("new name for test collection"), None)
            .await
            .is_err());

        //Test for setting new metadata. Should pass.
        assert!(collection
            .modify(
                None,
                Some(
                    json!({
                        "test": "test"
                    })
                    .as_object()
                    .unwrap()
                )
            )
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_get_from_collection() {
        let client = ChromaClient::new(Default::default());

        let collection = client
            .get_or_create_collection(TEST_COLLECTION, None)
            .await
            .unwrap();
        assert!(collection.count().await.is_ok());

        let get_query = GetOptions {
            ids: vec![],
            where_metadata: None,
            limit: None,
            offset: None,
            where_document: None,
            include: None,
        };
        let get_result = collection.get(get_query).await.unwrap();
        assert_eq!(get_result.ids.len(), collection.count().await.unwrap());
    }

    #[tokio::test]
    async fn test_add_to_collection() {
        let client = ChromaClient::new(Default::default());

        let collection = client
            .get_or_create_collection(TEST_COLLECTION, None)
            .await
            .unwrap();

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test1".into()],
            metadatas: None,
            documents: None,
            embeddings: None,
        };

        let response = collection.add(
            invalid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_err(),
            "Embeddings and documents cannot both be None"
        );

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.add(
            invalid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_err(),
            "IDs, embeddings, metadatas, and documents must all be the same length"
        );

        let valid_collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.add(
            valid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_ok(),
            "IDs, embeddings, metadatas, and documents must all be the same length"
        );

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.add(
            invalid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(response.await.is_err(), "Empty IDs not allowed");

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into(), "test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: Some(vec![vec![1.0, 2.0], vec![3.0, 4.0]]),
        };
        let response = collection.add(invalid_collection_entries, None);
        assert!(
            response.await.is_err(),
            "Expected IDs to be unique. Duplicates not allowed"
        );

        let collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.add(collection_entries, None);
        assert!(
            response.await.is_err(),
            "embedding_function cannot be None if documents are provided and embeddings are None"
        );

        let collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.add(collection_entries, Some(Box::new(MockEmbeddingProvider)));
        assert!(
            response.await.is_ok(),
            "Embeddings are computed by the embedding_function if embeddings are None and documents are provided"
        );
    }

    #[tokio::test]
    async fn test_upsert_collection() {
        let client = ChromaClient::new(Default::default());

        let collection = client
            .get_or_create_collection(TEST_COLLECTION, None)
            .await
            .unwrap();

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test1".into()],
            metadatas: None,
            documents: None,
            embeddings: None,
        };

        let response = collection.upsert(
            invalid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_err(),
            "Embeddings and documents cannot both be None"
        );

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.upsert(
            invalid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_err(),
            "IDs, embeddings, metadatas, and documents must all be the same length"
        );

        let valid_collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.upsert(
            valid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_ok(),
            "IDs, embeddings, metadatas, and documents must all be the same length"
        );

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.upsert(
            invalid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(response.await.is_err(), "Empty IDs not allowed");

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into(), "test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: Some(vec![vec![1.0, 2.0], vec![3.0, 4.0]]),
        };
        let response = collection.upsert(invalid_collection_entries, None);
        assert!(
            response.await.is_err(),
            "Expected IDs to be unique. Duplicates not allowed"
        );

        let collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.upsert(collection_entries, None);
        assert!(
            response.await.is_err(),
            "embedding_function cannot be None if documents are provided and embeddings are None"
        );

        let collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.upsert(collection_entries, Some(Box::new(MockEmbeddingProvider)));
        assert!(
            response.await.is_ok(),
            "Embeddings are computed by the embedding_function if embeddings are None and documents are provided"
        );
    }

    #[tokio::test]
    async fn test_update_collection() {
        let client = ChromaClient::new(Default::default());

        let collection = client
            .get_or_create_collection(TEST_COLLECTION, None)
            .await
            .unwrap();

        let valid_collection_entries = CollectionEntries {
            ids: vec!["test1".into()],
            metadatas: None,
            documents: None,
            embeddings: None,
        };

        let response = collection.update(
            valid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_ok(),
            "Embeddings and documents can both be None"
        );

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.update(
            invalid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_err(),
            "IDs, embeddings, metadatas, and documents must all be the same length"
        );

        let valid_collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.update(
            valid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(
            response.await.is_ok(),
            "IDs, embeddings, metadatas, and documents must all be the same length"
        );

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.update(
            invalid_collection_entries,
            Some(Box::new(MockEmbeddingProvider)),
        );
        assert!(response.await.is_err(), "Empty IDs not allowed");

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into(), "test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: Some(vec![vec![1.0, 2.0], vec![3.0, 4.0]]),
        };
        let response = collection.update(invalid_collection_entries, None);
        assert!(
            response.await.is_err(),
            "Expected IDs to be unique. Duplicates not allowed"
        );

        let collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.update(collection_entries, None);
        assert!(
            response.await.is_err(),
            "embedding_function cannot be None if documents are provided and embeddings are None"
        );

        let collection_entries = CollectionEntries {
            ids: vec!["test1".into(), "test2".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: None,
        };
        let response = collection.update(collection_entries, Some(Box::new(MockEmbeddingProvider)));
        assert!(
            response.await.is_ok(),
            "Embeddings are computed by the embedding_function if embeddings are None and documents are provided"
        );
    }

    #[tokio::test]
    async fn test_query_collection() {
        let client = ChromaClient::new(Default::default());

        let collection = client
            .get_or_create_collection(TEST_COLLECTION, None)
            .await
            .unwrap();
        assert!(collection.count().await.is_ok());

        let query = QueryOptions {
            query_texts: None,
            query_embeddings: None,
            where_metadata: None,
            where_document: None,
            n_results: None,
            include: None,
        };
        let query_result = collection.query(query, None);
        assert!(
            query_result.await.is_err(),
            "query_texts and query_embeddings cannot both be None"
        );

        let query = QueryOptions {
            query_texts: Some(vec![
                "Writing tests help me find bugs".into(),
                "Running them does not".into(),
            ]),
            query_embeddings: None,
            where_metadata: None,
            where_document: None,
            n_results: None,
            include: None,
        };
        let query_result = collection.query(query, Some(Box::new(MockEmbeddingProvider)));
        assert!(
            query_result.await.is_ok(),
            "query_embeddings will be computed from query_texts if embedding_function is provided"
        );

        let query = QueryOptions {
            query_texts: Some(vec![
                "Writing tests help me find bugs".into(),
                "Running them does not".into(),
            ]),
            query_embeddings: Some(vec![vec![0.0_f32; 768], vec![0.0_f32; 768]]),
            where_metadata: None,
            where_document: None,
            n_results: None,
            include: None,
        };
        let query_result = collection.query(query, Some(Box::new(MockEmbeddingProvider)));
        assert!(
            query_result.await.is_err(),
            "Both query_embeddings and query_texts cannot be provided"
        );

        let query = QueryOptions {
            query_texts: None,
            query_embeddings: Some(vec![vec![0.0_f32; 768], vec![0.0_f32; 768]]),
            where_metadata: None,
            where_document: None,
            n_results: None,
            include: None,
        };
        let query_result = collection.query(query, None);
        assert!(
            query_result.await.is_ok(),
            "Use provided query_embeddings if embedding_function is None"
        );
    }
}

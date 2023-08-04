use anyhow::bail;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashSet, sync::Arc};

use super::{
    api::APIClientV1,
    commons::{Documents, Embeddings, Metadata, Metadatas, Result},
    embeddings::EmbeddingFunction,
};

/// A collection representation for interacting with the associated ChromaDB collection.
#[derive(Deserialize, Debug)]
pub struct ChromaCollection {
    #[serde(skip)]
    pub(super) api: Arc<APIClientV1>,
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
    /// * If you provide duplicates in ids
    /// * If you provide empty ids
    /// * If you provide documents and don't provide an embedding function when embeddings is None
    /// * If you provide an embedding function and don't provide documents
    /// * If you provide both embeddings and embedding_function
    ///
    pub async fn add(
        &self,
        collection_entries: CollectionEntries,
        embedding_function: Option<Box<dyn EmbeddingFunction>>,
    ) -> Result<bool> {
        let collection_entries = validate(true, collection_entries, embedding_function).await?;

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
        let response = response.json::<bool>().await?;

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
    /// * If you provide duplicates in ids
    /// * If you provide empty ids
    /// * If you provide documents and don't provide an embedding function when embeddings is None
    /// * If you provide an embedding function and don't provide documents
    /// * If you provide both embeddings and embedding_function
    ///
    pub async fn upsert(
        &self,
        collection_entries: CollectionEntries,
        embedding_function: Option<Box<dyn EmbeddingFunction>>,
    ) -> Result<bool> {
        let collection_entries = validate(true, collection_entries, embedding_function).await?;

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
        let response = response.json::<bool>().await?;

        Ok(response)
    }

    /// Get embeddings and their associate data from the data store. If no ids or filter is provided returns all embeddings up to limit starting at offset.
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
    pub async fn get(
        &self,
        ids: Vec<&str>,
        where_metadata: Option<Value>,
        limit: Option<usize>,
        offset: Option<usize>,
        where_document: Option<Value>,
        include: Option<Vec<&str>>,
    ) -> Result<GetResult> {
        let json_body = json!({
            "ids": ids,
            "where": where_metadata,
            "limit": limit,
            "offset": offset,
            "where_document": where_document,
            "include": include.unwrap_or_default(), //Include cannot be null
        });
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
    /// * If you provide duplicates in ids
    /// * If you provide empty ids
    /// * If you provide documents and don't provide an embedding function when embeddings is None
    /// * If you provide an embedding function and don't provide documents
    /// * If you provide both embeddings and embedding_function
    ///
    pub async fn update(
        &self,
        collection_entries: CollectionEntries,
        embedding_function: Option<Box<dyn EmbeddingFunction>>,
    ) -> Result<bool> {
        let collection_entries = validate(false, collection_entries, embedding_function).await?;

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

    pub fn query() {}

    ///Get the first entries in the collection up to the limit
    ///
    /// # Arguments
    ///
    /// * `limit` - The number of entries to return.
    ///
    pub async fn peek(&self, limit: usize) -> Result<GetResult> {
        self.get(vec![], None, Some(limit), None, None, None).await
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
    pub metadatas: Option<Vec<Metadata>>,
    pub documents: Option<Documents>,
    pub embeddings: Option<Embeddings>,
}

async fn validate(
    require_embeddings_or_documents: bool,
    collection_entries: CollectionEntries,
    embedding_function: Option<Box<dyn EmbeddingFunction>>,
) -> Result<CollectionEntries> {
    let CollectionEntries {
        ids,
        embeddings,
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

    let mut embeddingss = Vec::new();
    if embeddings.is_none() && documents.is_some() && embedding_function.is_some() {
        embeddingss = embedding_function
            .unwrap()
            .embed(&documents.as_ref().unwrap())
            .await;
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

pub struct CollectionEntries {
    pub ids: Vec<String>,
    pub metadatas: Option<Vec<Metadata>>,
    pub documents: Option<Documents>,
    pub embeddings: Option<Embeddings>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::v1::{
        collection::CollectionEntries, embeddings::MockEmbeddingProvider, ChromaClient,
    };

    const TEST_COLLECTION: &str = "11-recipies-for-octopus";

    #[tokio::test]
    async fn test_create_collection() {
        let client = ChromaClient::new(Default::default());

        let collection = client
            .get_or_create_collection(TEST_COLLECTION, None)
            .await
            .unwrap();
        assert!(collection.count().await.is_ok());

        let collections = client.list_collections().await.unwrap();
        assert!(collections[0].count().await.is_ok());
    }

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

        let get_result = collection
            .get(vec![], None, None, None, None, None)
            .await
            .unwrap();
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

        let response = collection
            .add(
                invalid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_err(),
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
        let response = collection
            .add(
                invalid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_err(),
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
        let response = collection
            .add(
                valid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_ok(),
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
        let response = collection
            .add(
                invalid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(response.is_err(), "Empty IDs not allowed");

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into(), "test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: Some(vec![vec![1.0, 2.0], vec![3.0, 4.0]]),
        };
        let response = collection.add(invalid_collection_entries, None).await;
        assert!(
            response.is_err(),
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
        let response = collection.add(collection_entries, None).await;
        assert!(
            response.is_err(),
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
        let response = collection
            .add(collection_entries, Some(Box::new(MockEmbeddingProvider)))
            .await;
        assert!(
            response.is_ok(),
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

        let response = collection
            .upsert(
                invalid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_err(),
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
        let response = collection
            .upsert(
                invalid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_err(),
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
        let response = collection
            .upsert(
                valid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_ok(),
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
        let response = collection
            .upsert(
                invalid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(response.is_err(), "Empty IDs not allowed");

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into(), "test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: Some(vec![vec![1.0, 2.0], vec![3.0, 4.0]]),
        };
        let response = collection.upsert(invalid_collection_entries, None).await;
        assert!(
            response.is_err(),
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
        let response = collection.upsert(collection_entries, None).await;
        assert!(
            response.is_err(),
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
        let response = collection
            .upsert(collection_entries, Some(Box::new(MockEmbeddingProvider)))
            .await;
        assert!(
            response.is_ok(),
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

        let response = collection
            .update(
                valid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_ok(),
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
        let response = collection
            .update(
                invalid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_err(),
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
        let response = collection
            .update(
                valid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(
            response.is_ok(),
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
        let response = collection
            .update(
                invalid_collection_entries,
                Some(Box::new(MockEmbeddingProvider)),
            )
            .await;
        assert!(response.is_err(), "Empty IDs not allowed");

        let invalid_collection_entries = CollectionEntries {
            ids: vec!["test".into(), "test".into()],
            metadatas: None,
            documents: Some(vec![
                "Document content 1".into(),
                "Document content 2".into(),
            ]),
            embeddings: Some(vec![vec![1.0, 2.0], vec![3.0, 4.0]]),
        };
        let response = collection.update(invalid_collection_entries, None).await;
        assert!(
            response.is_err(),
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
        let response = collection.update(collection_entries, None).await;
        assert!(
            response.is_err(),
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
        let response = collection
            .update(collection_entries, Some(Box::new(MockEmbeddingProvider)))
            .await;
        assert!(
            response.is_ok(),
            "Embeddings are computed by the embedding_function if embeddings are None and documents are provided"
        );
    }
}

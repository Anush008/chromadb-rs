use serde::Deserialize;
use serde_json::{json, Map, Value};

use super::{
    api::APIClientV1,
    commons::{ChromaAPIError, Documents, Embeddings, Metadata},
};

#[derive(Deserialize, Debug)]
pub struct ChromaCollection {
    #[serde(skip)]
    pub(super) api: APIClientV1,
    pub(super) id: String,
    pub(super) metadata: Option<Metadata>,
    pub(super) name: String,
}

impl<'a> ChromaCollection {
    // Get the UUID of the collection.
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    // Get the name of the collection.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    // Get the metadata of the collection.
    pub fn metadata(&self) -> Option<&Map<String, Value>> {
        self.metadata.as_ref()
    }

    /// The total number of embeddings added to the database.
    pub async fn count(&self) -> Result<usize, ChromaAPIError> {
        let path = format!("/collections/{}/count", self.id);
        let response = self.api.get(&path).await?;
        let count = response
            .json::<usize>()
            .await
            .map_err(ChromaAPIError::error)?;
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
    /// * `ChromaAPIError` - If the collection name is invalid
    pub async fn modify(
        &self,
        name: Option<&str>,
        metadata: Option<&Metadata>,
    ) -> Result<(), ChromaAPIError> {
        let json_body = json!({
            "new_name": name,
            "new_metadata": metadata,
        });
        let path = format!("/collections/{}", self.id);
        self.api.put(&path, Some(json_body)).await?;
        Ok(())
    }

    pub fn add() {
        todo!()
    }

    pub fn upsert() {
        todo!()
    }

    /// Get embeddings and their associate data from the data store. If no ids or filter is provided returns all embeddings up to limit starting at offset.
    ///
    /// # Arguments
    ///
    /// * `ids` - The ids of the embeddings to get. Optional..
    /// * `where_metadata` - Used to filter results by metadata. E.g. `{ "$and": [{"foo": "bar"}, {"price": {"$gte": 4.20}}] }`. Optional.
    /// * `limit` - The maximum number of documents to return. Optional.
    /// * `offset` - The offset to start returning results from. Useful for paging results with limit. Optional.
    /// * `where_document` - Used to filter by the documents. E.g. {"$contains": "hello"}
    /// * `include` - A list of what to include in the results. Can contain "embeddings", "metadatas", "documents". Ids are always included. Defaults to ["metadatas", "documents"]. Optional.
    ///
    /// # Errors
    ///
    /// * `ChromaAPIError` - If the collection name is invalid
    pub async fn get(
        &self,
        ids: Vec<&str>,
        where_metadata: Option<Value>,
        limit: Option<usize>,
        offset: Option<usize>,
        where_document: Option<Value>,
        include: Option<Vec<&str>>,
    ) -> Result<GetResult, ChromaAPIError> {
        let json_body = json!({
            "ids": ids,
            "where": where_metadata,
            "limit": limit,
            "offset": offset,
            "where_document": where_document,
            "include": include.unwrap_or_default(),
        });
        let path = format!("/collections/{}/get", self.id);
        let response = self.api.post(&path, Some(json_body)).await?;
        let get_result = response
            .json::<GetResult>()
            .await
            .map_err(ChromaAPIError::error)?;
        Ok(get_result)
    }

    pub fn update() {
        todo!()
    }

    pub fn query() {}

    pub fn peek() {}

    pub fn delete() {}
}

#[derive(Deserialize, Debug)]
pub struct GetResult {
    pub ids: Vec<String>,
    pub metadatas: Option<Vec<Metadata>>,
    pub documents: Option<Documents>,
    pub embeddings: Option<Embeddings>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::v1::ChromaClient;

    const TEST_COLLECTION: &str = "9-recipies-for-octopus";

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
}

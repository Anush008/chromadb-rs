use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashSet;

use super::{api::APIClientV1, commons::ChromaAPIError};

pub type Metadata = Map<String, Value>;

#[derive(Deserialize, Debug)]
pub struct ChromaCollection {
    #[serde(skip)]
    pub(super) api: APIClientV1,
    pub(super) id: String,
    pub(super) metadata: Option<Metadata>,
    pub(super) name: String,
}

impl<'a> ChromaCollection {
    async fn _validate(
        require_embeddings_or_documents: bool,
        ids: Vec<&'a str>,
        embeddings: Option<Vec<Vec<f64>>>,
        metadatas: Option<Vec<Metadata>>,
        documents: Option<Vec<&'a str>>,
        embedding_function: impl Fn(Vec<&str>) -> Vec<Vec<f64>>,
    ) -> Result<(Vec<&'a str>, Vec<Vec<f64>>, Vec<Metadata>, Vec<&'a str>), String> {
        let mut embeddings = embeddings;
        let documents = documents;

        if require_embeddings_or_documents && embeddings.is_none() && documents.is_none() {
            return Err("Embeddings and documents cannot both be None".into());
        }

        if embeddings.is_none() && documents.is_some() {
            let documents_array = documents.clone().unwrap();
            embeddings = Some(embedding_function(documents_array));
        }

        if embeddings.is_none() {
            return Err("Embeddings is None but shouldn't be".into());
        }

        let embeddings_array = embeddings.unwrap();
        let metadatas_array = metadatas.unwrap_or_default();
        let documents_array = documents.unwrap_or_default();

        for id in &ids {
            if id.is_empty() {
                return Err("Found empty string".into());
            }
        }

        if embeddings_array.len() != ids.len()
            || metadatas_array.len() != ids.len()
            || documents_array.len() != ids.len()
        {
            return Err(
                "IDs, embeddings, metadatas, and documents must all be the same length".into(),
            );
        }

        let unique_ids: HashSet<_> = ids.iter().collect();
        if unique_ids.len() != ids.len() {
            let duplicate_ids: Vec<_> = ids
                .iter()
                .filter(|id| ids.iter().filter(|x| x == id).count() > 1)
                .collect();
            return Err(format!(
                "Expected IDs to be unique, found duplicates for: {:?}",
                duplicate_ids
            ));
        }

        Ok((ids, embeddings_array, metadatas_array, documents_array))
    }

    pub fn add() {
        todo!()
    }

    pub fn upsert() {
        todo!()
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

    pub fn modify() {
        todo!()
    }

    pub fn get() {
        todo!()
    }

    pub fn update() {
        todo!()
    }

    pub fn query() {}

    pub fn peek() {}

    pub fn delete() {}

    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn metadata(&self) -> Option<&Map<String, Value>> {
        self.metadata.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::v1::ChromaClient;

    const TEST_COLLECTION: &str = "8-recipies-for-octopus";

    #[tokio::test]
    async fn test_create_collection() {
        let client = ChromaClient::new(Default::default());

        let collection = client
            .create_collection(TEST_COLLECTION, None, true)
            .await
            .unwrap();
        assert_eq!(collection.count().await.unwrap(), 0);
    }
}

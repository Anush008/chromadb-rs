use super::commons::Result;

use serde_json::Value;
use std::collections::HashSet;

use super::{api::APIClientV1, embeddings::IEmbeddingFunction};

pub struct ChromaCollection<T: IEmbeddingFunction> {
    api: APIClientV1,
    i_embedding_function: Option<T>,
    pub id: String,
    pub metadata: Option<Value>,
    pub name: String,
}

impl<T: IEmbeddingFunction> ChromaCollection<T> {
    fn new(
        api: APIClientV1,
        i_embedding_function: Option<T>,
        id: String,
        metadata: Option<Value>,
        name: String,
    ) -> ChromaCollection<T> {
        ChromaCollection {
            api,
            id,
            metadata,
            name,
            i_embedding_function,
        }
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn set_metadata(&mut self, metadata: Option<Value>) {
        self.metadata = metadata;
    }

    async fn validate<'a>(
        require_embeddings_or_documents: bool,
        ids: Vec<&'a str>,
        embeddings: Option<Vec<Vec<f64>>>,
        metadatas: Option<Vec<Value>>,
        documents: Option<Vec<&'a str>>,
        embedding_function: Option<impl IEmbeddingFunction>,
    ) -> Result<(Vec<&'a str>, Vec<Vec<f64>>, Vec<Value>, Vec<&'a str>)> {
        let mut embeddings = embeddings;
        let documents = documents;

        if require_embeddings_or_documents {
            if embeddings.is_none() && documents.is_none() {
                anyhow::bail!("embeddings and documents cannot both be none");
            }
        }

        if embeddings.is_none() && documents.is_some() {
            let documents_array = documents.clone().unwrap();
            if let Some(embedding_function) = embedding_function {
                embeddings = Some(embedding_function.generate(documents_array).await);
            } else {
                anyhow::bail!("EmbeddingFunction is None. Please configure an embedding function");
            }
        }

        if embeddings.is_none() {
            anyhow::bail!("Embeddings is undefined but shouldn't be");
        }

        let embeddings_array = embeddings.unwrap();
        let metadatas_array = metadatas.unwrap_or_default();
        let documents_array = documents.unwrap_or_default();

        for id in &ids {
            if id.is_empty() {
                anyhow::bail!("Expected ids to be strings, found empty string");
            }
        }

        if embeddings_array.len() != ids.len()
            || metadatas_array.len() != ids.len()
            || documents_array.len() != ids.len()
        {
            anyhow::bail!("ids, embeddings, metadatas, and documents must all be the same length");
        }

        let unique_ids: HashSet<_> = ids.iter().collect();
        if unique_ids.len() != ids.len() {
            let duplicate_ids: Vec<_> = ids
                .iter()
                .filter(|id| ids.iter().filter(|x| x == id).count() > 1)
                .collect();
            anyhow::bail!(
                "Expected IDs to be unique, found duplicates for: {:?}",
                duplicate_ids
            );
        }

        Ok((ids, embeddings_array, metadatas_array, documents_array))
    }

    pub fn add() {
        todo!()
    }

    pub fn upsert() {
        todo!()
    }

    pub fn count() {
        todo!()
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
}
